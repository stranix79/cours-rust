// Labo 16 : tests de sondes, module par module, de bas en haut :
// config, format Prometheus, dépôt SQLite en mémoire, sonde réelle sur un
// port local, routes via oneshot (sans port), et la boucle de sondage.
// Aucun réseau extérieur : tout se passe sur 127.0.0.1.
// Lancé par : cargo test   (ou cargo test nom_du_test)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt; // .collect() sur un corps de réponse
use serde_json::{Value, json};
use tower::ServiceExt; // .oneshot() sur le routeur

use sondes::config::{Config, parse_duree, parser};
use sondes::depot::Depot;
use sondes::sonde::{Cible, Mesure, maintenant, sonder, sonder_tous};
use sondes::{Etat, api, boucle};

const CONFIG_EXEMPLE: &str = r#"
intervalle = "5s"
parallelisme = 4

[[cible]]
nom = "web"
adresse = "127.0.0.1:8000"
timeout = "500ms"

[[cible]]
nom = "ssh"
adresse = "10.0.0.2:22"
"#;

fn mesure(cible: &str, instant: i64, ok: bool, duree_ms: u32) -> Mesure {
    Mesure {
        cible: cible.to_string(),
        instant,
        ok,
        duree_ms,
        erreur: if ok {
            None
        } else {
            Some("connection refused".to_string())
        },
    }
}

fn cible(nom: &str, adresse: &str) -> Cible {
    Cible {
        nom: nom.to_string(),
        adresse: adresse.to_string(),
        timeout: Duration::from_millis(500),
    }
}

// ------------------------------------------------------------ config

#[test]
fn durees_lisibles() {
    assert_eq!(parse_duree("500ms"), Ok(Duration::from_millis(500)));
    assert_eq!(parse_duree("10s"), Ok(Duration::from_secs(10)));
    assert_eq!(parse_duree("2m"), Ok(Duration::from_secs(120)));
    assert_eq!(parse_duree("1h"), Ok(Duration::from_secs(3_600)));
    assert_eq!(
        parse_duree(" 3s "),
        Ok(Duration::from_secs(3)),
        "espaces autour"
    );
    for faux in ["", "10", "s", "10 s", "1.5s", "10j", "-3s", "0s"] {
        assert!(parse_duree(faux).is_err(), "{faux:?} devrait être refusée");
    }
    let message = parse_duree("10j").expect_err("unité inconnue");
    assert!(
        message.contains("\"j\""),
        "le message cite l'unité : {message}"
    );
}

#[test]
fn config_depuis_une_chaine() {
    let config = parser(CONFIG_EXEMPLE).expect("config valide");
    assert_eq!(
        config,
        Config {
            ecoute: "127.0.0.1:8080".to_string(),
            base: "sondes.db".to_string(),
            intervalle: Duration::from_secs(5),
            parallelisme: 4,
            cibles: vec![
                cible("web", "127.0.0.1:8000"),
                Cible {
                    timeout: Duration::from_secs(1), // le timeout par défaut
                    ..cible("ssh", "10.0.0.2:22")
                },
            ],
        }
    );
}

#[test]
fn config_valeurs_par_defaut() {
    let config = parser("[[cible]]\nnom = \"a\"\nadresse = \"h:1\"\n").expect("config minimale");
    assert_eq!(config.intervalle, Duration::from_secs(10));
    assert_eq!(config.parallelisme, 8);
    assert_eq!(config.ecoute, "127.0.0.1:8080");
}

#[test]
fn config_refusees() {
    let cas = [
        (
            "intervale = \"5s\"\n[[cible]]\nnom = \"a\"\nadresse = \"h:1\"",
            "unknown field",
        ),
        ("intervalle = \"5s\"", "aucune cible"),
        (
            "parallelisme = 0\n[[cible]]\nnom = \"a\"\nadresse = \"h:1\"",
            "parallelisme",
        ),
        (
            "intervalle = \"5 s\"\n[[cible]]\nnom = \"a\"\nadresse = \"h:1\"",
            "intervalle",
        ),
        (
            "[[cible]]\nnom = \"a\"\nadresse = \"h:1\"\n[[cible]]\nnom = \"a\"\nadresse = \"h:2\"",
            "en double",
        ),
        (
            "[[cible]]\nnom = \"a\"\nadresse = \"sans-port\"",
            "sans port",
        ),
        ("[[cible]]\nnom = \"a\"\nadresse = \"h:99999\"", "sans port"),
        (
            "intervalle = \"1s\"\n[[cible]]\nnom = \"a\"\nadresse = \"h:1\"\ntimeout = \"2s\"",
            "plus long",
        ),
        ("[[cible]]\nnom = \"  \"\nadresse = \"h:1\"", "nom vide"),
    ];
    for (texte, attendu) in cas {
        let erreur = parser(texte).expect_err(texte);
        // {:#} : toute la chaîne des causes, pour voir aussi l'erreur de toml.
        let message = format!("{erreur:#}");
        assert!(
            message.contains(attendu),
            "pour {texte:?}, message : {message}"
        );
    }
}

// ------------------------------------------------------- Prometheus

#[test]
fn prometheus_a_l_espace_pres() {
    let mesures = [
        mesure("web", 1, true, 12),
        mesure(r#"pg "prod"\1"#, 1, false, 500),
    ];
    assert_eq!(
        api::format_prometheus(&mesures, 42),
        "# HELP sondes_up 1 si la cible a répondu à la dernière sonde, 0 sinon.\n\
         # TYPE sondes_up gauge\n\
         sondes_up{cible=\"web\"} 1\n\
         sondes_up{cible=\"pg \\\"prod\\\"\\\\1\"} 0\n\
         # HELP sondes_duree_ms Durée de la dernière sonde, en millisecondes.\n\
         # TYPE sondes_duree_ms gauge\n\
         sondes_duree_ms{cible=\"web\"} 12\n\
         sondes_duree_ms{cible=\"pg \\\"prod\\\"\\\\1\"} 500\n\
         # HELP sondes_tours_total Tours de sondage effectués depuis le démarrage.\n\
         # TYPE sondes_tours_total counter\n\
         sondes_tours_total 42\n"
    );
}

#[test]
fn prometheus_sans_mesure() {
    // Avant le premier tour : les en-têtes, aucune série, le compteur à 0.
    let texte = api::format_prometheus(&[], 0);
    assert!(!texte.contains("sondes_up{"));
    assert!(texte.ends_with("sondes_tours_total 0\n"), "{texte}");
}

#[test]
fn labels_echappes() {
    assert_eq!(api::echapper_label("simple"), "simple");
    assert_eq!(api::echapper_label("a\"b"), "a\\\"b");
    assert_eq!(api::echapper_label("a\\b"), "a\\\\b");
    assert_eq!(api::echapper_label("a\nb"), "a\\nb");
}

// ------------------------------------------------------------ dépôt

#[test]
fn depot_en_memoire() {
    let mut depot = Depot::en_memoire().expect("SQLite en mémoire");
    assert_eq!(depot.dernieres(None, 10).expect("dernieres"), []);
    assert_eq!(depot.taux_disponibilite("web", 0).expect("taux"), None);

    depot
        .enregistrer(&[mesure("web", 100, true, 3), mesure("pg", 100, false, 500)])
        .expect("tour 1");
    depot
        .enregistrer(&[mesure("web", 110, false, 500), mesure("pg", 110, true, 4)])
        .expect("tour 2");
    depot
        .enregistrer(&[mesure("web", 120, true, 2), mesure("pg", 120, true, 5)])
        .expect("tour 3");

    // La plus récente d'abord, limitée à n, filtrée par cible.
    let web = depot.dernieres(Some("web"), 2).expect("dernieres web");
    assert_eq!(
        web,
        [mesure("web", 120, true, 2), mesure("web", 110, false, 500)]
    );
    assert_eq!(depot.dernieres(None, 100).expect("toutes").len(), 6);
    assert_eq!(depot.dernieres(Some("inconnue"), 10).expect("inconnue"), []);
    // L'erreur (NULL ou texte) fait l'aller-retour.
    assert_eq!(web[1].erreur.as_deref(), Some("connection refused"));

    // Taux : 2 réussites sur 3 pour web, 1 sur 1 depuis 120.
    let taux = depot
        .taux_disponibilite("web", 0)
        .expect("taux")
        .expect("des mesures");
    assert!((taux - 2.0 / 3.0).abs() < 1e-9, "taux {taux}");
    assert_eq!(
        depot.taux_disponibilite("web", 120).expect("taux"),
        Some(1.0)
    );
    assert_eq!(
        depot.taux_disponibilite("web", 121).expect("taux"),
        None,
        "rien depuis 121"
    );
    assert_eq!(
        depot.taux_disponibilite("pg", 110).expect("taux"),
        Some(1.0)
    );
}

// ------------------------------------------------------------ sonde

#[test]
fn sonde_reelle_sur_un_port_local() {
    // Un port ouvert : un TcpListener qui écoute, sans même accepter. Le
    // noyau termine la poignée de main TCP tout seul.
    let ecoute = std::net::TcpListener::bind("127.0.0.1:0").expect("écoute");
    let ouvert = ecoute.local_addr().expect("adresse").to_string();
    // Un port fermé : on en réserve un, puis on le libère.
    let ferme = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").expect("écoute");
        l.local_addr().expect("adresse").to_string()
    };
    let arret = AtomicBool::new(false);

    let m = sonder(&cible("ouvert", &ouvert), &arret);
    assert!(m.ok, "port ouvert : {m:?}");
    assert_eq!(m.erreur, None);
    assert_eq!(m.cible, "ouvert");
    assert!((m.instant - maintenant()).abs() <= 2, "instant récent");

    let m = sonder(&cible("ferme", &ferme), &arret);
    assert!(!m.ok, "port fermé : {m:?}");
    assert!(m.erreur.is_some());

    arret.store(true, Ordering::Relaxed);
    let m = sonder(&cible("ouvert", &ouvert), &arret);
    assert_eq!(m.erreur.as_deref(), Some("annulé"));
}

#[test]
fn sonder_tous_garde_l_ordre() {
    let ecoute = std::net::TcpListener::bind("127.0.0.1:0").expect("écoute");
    let ouvert = ecoute.local_addr().expect("adresse").to_string();
    let cibles: Vec<Cible> = (0..6)
        .map(|i| {
            if i % 2 == 0 {
                cible(&format!("c{i}"), &ouvert)
            } else {
                cible(&format!("c{i}"), "127.0.0.1:1")
            }
        })
        .collect();
    let mesures = sonder_tous(&cibles, 3, &Arc::new(AtomicBool::new(false)));
    let noms: Vec<&str> = mesures.iter().map(|m| m.cible.as_str()).collect();
    assert_eq!(noms, ["c0", "c1", "c2", "c3", "c4", "c5"]);
    let etats: Vec<bool> = mesures.iter().map(|m| m.ok).collect();
    assert_eq!(etats, [true, false, true, false, true, false]);
}

// ------------------------------------------------------------ API

fn etat_de_test() -> Arc<Etat> {
    let config = parser(CONFIG_EXEMPLE).expect("config");
    Etat::nouveau(config, Depot::en_memoire().expect("dépôt"))
}

async fn appeler(etat: &Arc<Etat>, uri: &str) -> (StatusCode, String, String) {
    let requete = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("requête");
    let reponse = api::routeur(Arc::clone(etat))
        .oneshot(requete)
        .await
        .expect("le routeur ne renvoie jamais d'erreur");
    let statut = reponse.status();
    let type_contenu = reponse
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let octets = reponse
        .into_body()
        .collect()
        .await
        .expect("corps")
        .to_bytes();
    (
        statut,
        type_contenu,
        String::from_utf8(octets.to_vec()).expect("UTF-8"),
    )
}

async fn appeler_json(etat: &Arc<Etat>, uri: &str) -> (StatusCode, Value) {
    let (statut, _, texte) = appeler(etat, uri).await;
    let valeur = serde_json::from_str(&texte)
        .unwrap_or_else(|e| panic!("JSON attendu, reçu {texte:?} ({e})"));
    (statut, valeur)
}

#[tokio::test]
async fn healthz() {
    let (statut, corps) = appeler_json(&etat_de_test(), "/healthz").await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["statut"], "ok");
}

#[tokio::test]
async fn status_avant_le_premier_tour() {
    let (statut, corps) = appeler_json(&etat_de_test(), "/api/status").await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["tours"], 0);
    assert_eq!(
        corps["cibles"][0],
        json!({
            "nom": "web", "adresse": "127.0.0.1:8000", "ok": null, "duree_ms": null,
            "erreur": null, "instant": null, "disponibilite_24h": null
        })
    );
    assert_eq!(corps["cibles"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn status_apres_trois_tours() {
    let etat = etat_de_test();
    let t = maintenant();
    etat.enregistrer(vec![
        mesure("web", t - 20, true, 3),
        mesure("ssh", t - 20, false, 1000),
    ])
    .expect("tour 1");
    etat.enregistrer(vec![
        mesure("web", t - 10, false, 500),
        mesure("ssh", t - 10, false, 1000),
    ])
    .expect("tour 2");
    etat.enregistrer(vec![
        mesure("web", t, true, 2),
        mesure("ssh", t, false, 1000),
    ])
    .expect("tour 3");

    let (statut, corps) = appeler_json(&etat, "/api/status").await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["tours"], 3);
    let web = &corps["cibles"][0];
    assert_eq!(web["nom"], "web", "l'ordre de la configuration");
    assert_eq!(web["ok"], true);
    assert_eq!(web["duree_ms"], 2);
    assert_eq!(web["instant"], t);
    let dispo = web["disponibilite_24h"].as_f64().expect("un nombre");
    assert!((dispo - 2.0 / 3.0).abs() < 1e-9, "disponibilité {dispo}");
    let ssh = &corps["cibles"][1];
    assert_eq!(ssh["ok"], false);
    assert_eq!(ssh["erreur"], "connection refused");
    assert_eq!(ssh["disponibilite_24h"], 0.0);
}

#[tokio::test]
async fn historique_filtre_et_limite() {
    let etat = etat_de_test();
    for i in 0..5 {
        etat.enregistrer(vec![
            mesure("web", 100 + i, true, 1),
            mesure("ssh", 100 + i, false, 9),
        ])
        .expect("tour");
    }
    let (statut, corps) = appeler_json(&etat, "/api/historique?cible=web&n=2").await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(
        corps,
        json!([
            { "cible": "web", "instant": 104, "ok": true, "duree_ms": 1, "erreur": null },
            { "cible": "web", "instant": 103, "ok": true, "duree_ms": 1, "erreur": null }
        ])
    );
    let (_, corps) = appeler_json(&etat, "/api/historique").await;
    assert_eq!(
        corps.as_array().map(Vec::len),
        Some(10),
        "toutes les cibles, 20 au plus"
    );

    let (statut, corps) = appeler_json(&etat, "/api/historique?cible=inconnue").await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    assert_eq!(corps, json!({ "erreur": "cible \"inconnue\" inconnue" }));

    let (statut, corps) = appeler_json(&etat, "/api/historique?n=abc").await;
    assert_eq!(statut, StatusCode::BAD_REQUEST);
    assert!(corps["erreur"].is_string());
}

#[tokio::test]
async fn metrics_en_texte() {
    let etat = etat_de_test();
    etat.enregistrer(vec![
        mesure("ssh", 1, false, 1000),
        mesure("web", 1, true, 3),
    ])
    .expect("tour");
    let (statut, type_contenu, corps) = appeler(&etat, "/metrics").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(
        type_contenu.starts_with("text/plain; version=0.0.4"),
        "{type_contenu}"
    );
    // L'ordre de la configuration (web avant ssh), pas celui de l'enregistrement.
    let web = corps.find("sondes_up{cible=\"web\"} 1").expect("série web");
    let ssh = corps.find("sondes_up{cible=\"ssh\"} 0").expect("série ssh");
    assert!(web < ssh, "{corps}");
    assert!(corps.ends_with("sondes_tours_total 1\n"), "{corps}");
}

#[tokio::test]
async fn route_et_methode_inconnues() {
    let etat = etat_de_test();
    let (statut, _, _) = appeler(&etat, "/nulle-part").await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    let requete = Request::builder()
        .method("POST")
        .uri("/api/status")
        .body(Body::empty())
        .expect("requête");
    let reponse = api::routeur(etat).oneshot(requete).await.expect("réponse");
    assert_eq!(reponse.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// ------------------------------------------------------- la boucle

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boucle_sonde_puis_s_arrete() {
    let ecoute = std::net::TcpListener::bind("127.0.0.1:0").expect("écoute");
    let adresse = ecoute.local_addr().expect("adresse");
    let texte = format!(
        "intervalle = \"50ms\"\n[[cible]]\nnom = \"local\"\nadresse = \"{adresse}\"\ntimeout = \"50ms\"\n"
    );
    let etat = Etat::nouveau(
        parser(&texte).expect("config"),
        Depot::en_memoire().expect("dépôt"),
    );

    let (arreter, arret) = tokio::sync::oneshot::channel::<()>();
    let tache = tokio::spawn(boucle(Arc::clone(&etat), async {
        let _ = arret.await;
    }));
    // Un tour tout de suite, puis un toutes les 50 ms : en 180 ms, trois ou
    // quatre (au moins deux sur une machine très chargée).
    tokio::time::sleep(Duration::from_millis(180)).await;
    arreter.send(()).expect("la boucle écoute encore");
    tokio::time::timeout(Duration::from_secs(1), tache)
        .await
        .expect("la boucle ne s'est pas arrêtée en 1 s")
        .expect("tâche");

    let tours = etat.tours.load(Ordering::Relaxed);
    assert!((2..=5).contains(&tours), "{tours} tours en 180 ms");
    let historique = etat
        .depot
        .lock()
        .expect("verrou")
        .dernieres(Some("local"), 100)
        .expect("historique");
    assert_eq!(u64::try_from(historique.len()).expect("taille"), tours);
    assert!(
        historique.iter().all(|m| m.ok),
        "la cible écoute : {historique:?}"
    );
    // Après l'arrêt, plus aucun tour.
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(etat.tours.load(Ordering::Relaxed), tours);
}
