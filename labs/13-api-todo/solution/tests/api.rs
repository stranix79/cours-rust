// Labo 13 : tests de l'API. Deux familles :
// - sans port : le routeur est un service tower, on lui envoie une requête
//   avec oneshot() et on lit la réponse, sans réseau ni serveur ;
// - de bout en bout : un vrai serveur sur 127.0.0.1:0 et le client reqwest.
// Lancé par : cargo test   (ou cargo test nom_du_test)

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt; // .collect() sur un corps de réponse
use serde_json::{Value, json};
use tower::ServiceExt; // .oneshot() sur le routeur

use todo_api::{Etat, Todo, format_tableau, recuperer_todos, routeur, servir};

/// Envoie une requête au routeur sans ouvrir de port et rend le statut et le
/// corps (texte brut). Le routeur est reconstruit à chaque appel autour du
/// même Arc<Etat> : c'est l'état qui porte les données, pas le routeur.
async fn appeler(
    etat: &Arc<Etat>,
    methode: &str,
    uri: &str,
    corps: Option<&str>,
) -> (StatusCode, String) {
    let mut requete = Request::builder().method(methode).uri(uri);
    if corps.is_some() {
        requete = requete.header(header::CONTENT_TYPE, "application/json");
    }
    let requete = requete
        .body(corps.map_or_else(Body::empty, |c| Body::from(c.to_string())))
        .expect("requête");
    let reponse = routeur(Arc::clone(etat))
        .oneshot(requete)
        .await
        .expect("le routeur ne renvoie jamais d'erreur");
    let statut = reponse.status();
    let octets = reponse
        .into_body()
        .collect()
        .await
        .expect("corps")
        .to_bytes();
    (statut, String::from_utf8(octets.to_vec()).expect("UTF-8"))
}

/// Comme appeler(), mais décode le corps en JSON.
async fn appeler_json(
    etat: &Arc<Etat>,
    methode: &str,
    uri: &str,
    corps: Option<&str>,
) -> (StatusCode, Value) {
    let (statut, texte) = appeler(etat, methode, uri, corps).await;
    let valeur = serde_json::from_str(&texte)
        .unwrap_or_else(|e| panic!("JSON attendu, reçu {texte:?} ({e})"));
    (statut, valeur)
}

/// Un état avec deux tâches déjà créées par l'API.
async fn etat_avec_deux_todos() -> Arc<Etat> {
    let etat = Etat::nouveau();
    for titre in ["sauvegarder pg-01", "renouveler le certificat"] {
        let corps = json!({ "titre": titre }).to_string();
        let (statut, _) = appeler(&etat, "POST", "/todos", Some(&corps)).await;
        assert_eq!(statut, StatusCode::CREATED, "création de {titre}");
    }
    etat
}

#[tokio::test]
async fn healthz_repond_ok() {
    let (statut, corps) = appeler_json(&Etat::nouveau(), "GET", "/healthz", None).await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps, json!({ "statut": "ok" }));
}

#[tokio::test]
async fn liste_vide_est_un_tableau_vide() {
    let (statut, corps) = appeler(&Etat::nouveau(), "GET", "/todos", None).await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps, "[]");
}

#[tokio::test]
async fn creer_renvoie_201_et_la_tache() {
    let etat = Etat::nouveau();
    let (statut, corps) = appeler_json(
        &etat,
        "POST",
        "/todos",
        Some(r#"{"titre":"  sauvegarder pg-01  "}"#),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);
    // Le titre est nettoyé de ses espaces, l'id commence à 1.
    assert_eq!(
        corps,
        json!({ "id": 1, "titre": "sauvegarder pg-01", "fait": false })
    );
    let (_, corps) = appeler_json(&etat, "POST", "/todos", Some(r#"{"titre":"b"}"#)).await;
    assert_eq!(corps["id"], 2, "le deuxième id est 2");
}

#[tokio::test]
async fn creer_titre_vide_renvoie_400() {
    let (statut, corps) = appeler_json(
        &Etat::nouveau(),
        "POST",
        "/todos",
        Some(r#"{"titre":"   "}"#),
    )
    .await;
    assert_eq!(statut, StatusCode::BAD_REQUEST);
    assert_eq!(corps, json!({ "erreur": "le titre est obligatoire" }));
}

#[tokio::test]
async fn creer_json_invalide_renvoie_400_en_json() {
    let etat = Etat::nouveau();
    for corps in ["pas du json", r#"{"title":"faute de frappe"}"#, "{}"] {
        let (statut, valeur) = appeler_json(&etat, "POST", "/todos", Some(corps)).await;
        assert_eq!(statut, StatusCode::BAD_REQUEST, "corps {corps:?}");
        let message = valeur["erreur"].as_str().expect("champ erreur");
        assert!(message.starts_with("JSON invalide"), "message : {message}");
    }
    // Rien n'a été créé.
    let (_, corps) = appeler(&etat, "GET", "/todos", None).await;
    assert_eq!(corps, "[]");
}

#[tokio::test]
async fn liste_dans_l_ordre_de_creation() {
    let etat = etat_avec_deux_todos().await;
    let (statut, corps) = appeler(&etat, "GET", "/todos", None).await;
    assert_eq!(statut, StatusCode::OK);
    let todos: Vec<Todo> = serde_json::from_str(&corps).expect("liste de Todo");
    let ids: Vec<u64> = todos.iter().map(|t| t.id).collect();
    assert_eq!(ids, [1, 2]);
    assert_eq!(todos[1].titre, "renouveler le certificat");
}

#[tokio::test]
async fn lire_existant_inconnu_et_invalide() {
    let etat = etat_avec_deux_todos().await;

    let (statut, corps) = appeler_json(&etat, "GET", "/todos/2", None).await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["titre"], "renouveler le certificat");

    let (statut, corps) = appeler_json(&etat, "GET", "/todos/42", None).await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    assert_eq!(corps, json!({ "erreur": "todo 42 introuvable" }));

    let (statut, corps) = appeler_json(&etat, "GET", "/todos/abc", None).await;
    assert_eq!(statut, StatusCode::BAD_REQUEST);
    let message = corps["erreur"].as_str().expect("champ erreur");
    assert!(message.starts_with("id invalide"), "message : {message}");
}

#[tokio::test]
async fn modifier_fait_puis_titre() {
    let etat = etat_avec_deux_todos().await;

    // Seulement fait : le titre ne bouge pas.
    let (statut, corps) = appeler_json(&etat, "PUT", "/todos/1", Some(r#"{"fait":true}"#)).await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(
        corps,
        json!({ "id": 1, "titre": "sauvegarder pg-01", "fait": true })
    );

    // Seulement le titre : fait reste vrai.
    let (statut, corps) = appeler_json(
        &etat,
        "PUT",
        "/todos/1",
        Some(r#"{"titre":"sauvegarder pg-01 et pg-02"}"#),
    )
    .await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["titre"], "sauvegarder pg-01 et pg-02");
    assert_eq!(corps["fait"], true);

    // La modification est bien dans l'état, pas seulement dans la réponse.
    let (_, corps) = appeler_json(&etat, "GET", "/todos/1", None).await;
    assert_eq!(corps["titre"], "sauvegarder pg-01 et pg-02");
}

#[tokio::test]
async fn modifier_erreurs() {
    let etat = etat_avec_deux_todos().await;

    let (statut, corps) = appeler_json(&etat, "PUT", "/todos/1", Some(r#"{"titre":""}"#)).await;
    assert_eq!(statut, StatusCode::BAD_REQUEST);
    assert_eq!(corps, json!({ "erreur": "le titre est obligatoire" }));

    let (statut, corps) = appeler_json(&etat, "PUT", "/todos/99", Some(r#"{"fait":true}"#)).await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    assert_eq!(corps, json!({ "erreur": "todo 99 introuvable" }));

    let (statut, _) = appeler_json(&etat, "PUT", "/todos/1", Some(r#"{"fait":"oui"}"#)).await;
    assert_eq!(statut, StatusCode::BAD_REQUEST, "fait doit être un booléen");

    // Le titre vide n'a rien changé.
    let (_, corps) = appeler_json(&etat, "GET", "/todos/1", None).await;
    assert_eq!(corps["titre"], "sauvegarder pg-01");
}

#[tokio::test]
async fn supprimer_204_puis_404() {
    let etat = etat_avec_deux_todos().await;

    let (statut, corps) = appeler(&etat, "DELETE", "/todos/2", None).await;
    assert_eq!(statut, StatusCode::NO_CONTENT);
    assert!(corps.is_empty(), "204 sans corps, reçu {corps:?}");

    let (statut, _) = appeler(&etat, "GET", "/todos/2", None).await;
    assert_eq!(statut, StatusCode::NOT_FOUND);

    let (statut, corps) = appeler_json(&etat, "DELETE", "/todos/2", None).await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    assert_eq!(corps, json!({ "erreur": "todo 2 introuvable" }));

    // L'autre tâche est toujours là, et un nouvel id ne réutilise pas le 2.
    let (_, corps) = appeler_json(&etat, "POST", "/todos", Some(r#"{"titre":"c"}"#)).await;
    assert_eq!(corps["id"], 3);
}

#[tokio::test]
async fn methode_et_route_inconnues() {
    let etat = Etat::nouveau();
    let (statut, _) = appeler(&etat, "PATCH", "/todos", None).await;
    assert_eq!(statut, StatusCode::METHOD_NOT_ALLOWED);
    let (statut, _) = appeler(&etat, "GET", "/nulle-part", None).await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
}

// multi_thread : les cinquante tâches tournent vraiment en parallèle sur
// plusieurs threads, comme les connexions d'un vrai serveur.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn creations_concurrentes_ids_uniques() {
    let etat = Etat::nouveau();
    let mut taches = Vec::new();
    for i in 0..50 {
        let etat = Arc::clone(&etat);
        taches.push(tokio::spawn(async move {
            let corps = json!({ "titre": format!("tâche {i}") }).to_string();
            appeler(&etat, "POST", "/todos", Some(&corps)).await.0
        }));
    }
    for t in taches {
        assert_eq!(t.await.expect("tâche"), StatusCode::CREATED);
    }
    let (_, corps) = appeler(&etat, "GET", "/todos", None).await;
    let todos: Vec<Todo> = serde_json::from_str(&corps).expect("liste");
    let mut ids: Vec<u64> = todos.iter().map(|t| t.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, (1..=50).collect::<Vec<u64>>());
}

#[test]
fn format_tableau_aligne_les_colonnes() {
    let todos = [
        Todo {
            id: 1,
            titre: "sauvegarder pg-01".into(),
            fait: true,
        },
        Todo {
            id: 12,
            titre: "renouveler le certificat".into(),
            fait: false,
        },
    ];
    assert_eq!(
        format_tableau(&todos),
        "ID  FAIT  TITRE\n1   [x]   sauvegarder pg-01\n12  [ ]   renouveler le certificat\n"
    );
    assert_eq!(format_tableau(&[]), "ID  FAIT  TITRE\n");
}

#[tokio::test]
async fn bout_en_bout_sur_un_vrai_port() {
    // Port 0 : le système choisit un port libre, local_addr() dit lequel.
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("écoute");
    let base = format!("http://{}", ecoute.local_addr().expect("adresse"));

    // Le signal d'arrêt est un canal oneshot : le serveur s'arrête quand on
    // envoie (ou quand l'émetteur est détruit).
    let (arreter, arret) = tokio::sync::oneshot::channel::<()>();
    let serveur = tokio::spawn(servir(ecoute, Etat::nouveau(), async {
        let _ = arret.await;
    }));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client");
    let reponse = client
        .post(format!("{base}/todos"))
        .json(&json!({ "titre": "vider /var/log sur web-02" }))
        .send()
        .await
        .expect("POST");
    assert_eq!(reponse.status(), reqwest::StatusCode::CREATED);

    let todos = recuperer_todos(&client, &base).await.expect("liste");
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].titre, "vider /var/log sur web-02");
    assert!(format_tableau(&todos).contains("[ ]   vider /var/log sur web-02"));

    // Un statut d'erreur doit devenir une Err côté client.
    let erreur = recuperer_todos(&client, &format!("{base}/nulle-part"))
        .await
        .expect_err("404 attendu");
    assert!(erreur.to_string().contains("404"), "erreur : {erreur}");

    // Arrêt propre : servir() doit rendre la main vite.
    arreter.send(()).expect("le serveur écoute encore");
    let fin = tokio::time::timeout(Duration::from_secs(1), serveur)
        .await
        .expect("le serveur ne s'est pas arrêté en 1 s");
    fin.expect("tâche").expect("servir sans erreur");
}
