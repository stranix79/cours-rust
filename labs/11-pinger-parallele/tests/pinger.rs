// Labo 11 : les tests d'intégration de pinger. Identiques dans le squelette
// et dans la solution. Ils montent un vrai serveur TCP local sur un port
// libre : déterministes, sans réseau extérieur, moins d'une seconde.

use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, Instant};

use pinger::{
    Cible, ErreurCible, Resultat, TIMEOUT_DEFAUT, duree_courte, format_tableau, parse_cible,
    sonder, sonder_tous,
};

/// Ouvre un vrai serveur TCP sur un port libre de 127.0.0.1, qui accepte et
/// referme chaque connexion dans un thread, et rend le port. Le thread vit
/// jusqu'à la fin du processus de test : c'est sans conséquence ici.
fn serveur_local() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    let port = l.local_addr().unwrap().port();
    thread::spawn(move || {
        for conn in l.incoming() {
            drop(conn); // accepter puis refermer aussitôt
        }
    });
    port
}

/// Rend un port sur lequel personne n'écoute : on en réserve un, puis on le
/// libère aussitôt en lâchant le listener.
fn port_ferme() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    l.local_addr().unwrap().port()
}

fn cible(hote: &str, port: u16) -> Cible {
    Cible {
        hote: hote.to_string(),
        port,
        timeout: None,
    }
}

fn pas_d_arret() -> AtomicBool {
    AtomicBool::new(false)
}

#[test]
fn parse_cible_les_sept_cas() {
    let cas: Vec<(&str, &str, Result<Cible, ErreurCible>)> = vec![
        ("hôte et port", "db1:5432", Ok(cible("db1", 5432))),
        ("adresse IPv4", "127.0.0.1:22", Ok(cible("127.0.0.1", 22))),
        ("adresse IPv6", "[::1]:443", Ok(cible("::1", 443))),
        ("sans port", "db1", Err(ErreurCible::SansPort)),
        (
            "port non numérique",
            "db1:pg",
            Err(ErreurCible::PortInvalide("pg".to_string())),
        ),
        ("port zéro", "db1:0", Err(ErreurCible::PortHorsPlage(0))),
        (
            "port trop grand",
            "db1:70000",
            Err(ErreurCible::PortHorsPlage(70000)),
        ),
    ];
    for (nom, entree, attendu) in cas {
        assert_eq!(parse_cible(entree), attendu, "cas « {nom} » : {entree:?}");
    }
}

#[test]
fn parse_cible_hote_vide() {
    assert_eq!(parse_cible(":80"), Err(ErreurCible::HoteVide));
}

#[test]
fn erreur_cible_affichable() {
    // Display est implémenté : le message parle du port fautif.
    let texte = ErreurCible::PortHorsPlage(70000).to_string();
    assert!(texte.contains("70000"), "message : {texte}");
}

#[test]
fn cible_display() {
    assert_eq!(cible("db1", 5432).to_string(), "db1:5432");
    assert_eq!(cible("::1", 22).to_string(), "[::1]:22");
}

#[test]
fn sonder_local_repond() {
    let port = serveur_local();
    let c = cible("127.0.0.1", port);
    let r = sonder(&c, &pas_d_arret());
    assert!(r.ok, "attendu OK, obtenu {r:?}");
    assert_eq!(r.erreur, None);
    assert_eq!(r.cible, c, "la cible doit être recopiée dans le résultat");
    assert!(
        r.duree > Duration::ZERO && r.duree < Duration::from_secs(1),
        "durée incohérente : {:?}",
        r.duree
    );
}

#[test]
fn sonder_port_ferme() {
    let c = cible("127.0.0.1", port_ferme());
    let debut = Instant::now();
    let r = sonder(&c, &pas_d_arret());
    assert!(!r.ok, "attendu KO, obtenu {r:?}");
    assert!(r.erreur.is_some(), "un KO doit porter un message");
    assert!(
        debut.elapsed() < Duration::from_secs(1),
        "un port fermé répond « connection refused » tout de suite"
    );
}

#[test]
fn sonder_arret_deja_leve() {
    let arret = AtomicBool::new(true); // levé AVANT la sonde
    let mut c = cible("192.0.2.1", 80); // adresse de documentation : ne répond jamais
    c.timeout = Some(Duration::from_secs(5));
    let debut = Instant::now();
    let r = sonder(&c, &arret);
    assert!(!r.ok);
    assert_eq!(r.erreur.as_deref(), Some("annulé"));
    assert!(
        debut.elapsed() < Duration::from_millis(500),
        "une sonde annulée doit rendre la main tout de suite, a pris {:?}",
        debut.elapsed()
    );
}

#[test]
fn sonder_tous_ordre_et_parallelisme() {
    let ouvert = serveur_local();
    let ferme = port_ferme();
    let cibles = vec![
        cible("127.0.0.1", ouvert),
        cible("127.0.0.1", ferme),
        cible("127.0.0.1", ouvert),
        cible("127.0.0.1", ferme),
        cible("127.0.0.1", ouvert),
    ];
    for p in [1, 2, 10, 0] {
        let res = sonder_tous(&cibles, p, Arc::new(pas_d_arret()));
        assert_eq!(res.len(), cibles.len(), "parallélisme {p}");
        for (i, r) in res.iter().enumerate() {
            assert_eq!(
                r.cible, cibles[i],
                "parallélisme {p} : résultat {i} (ordre non respecté)"
            );
            let veut_ok = cibles[i].port == ouvert;
            assert_eq!(
                r.ok, veut_ok,
                "parallélisme {p} : cible {} : {r:?}",
                r.cible
            );
        }
    }
}

#[test]
fn sonder_tous_vide() {
    let res = sonder_tous(&[], 4, Arc::new(pas_d_arret()));
    assert!(
        res.is_empty(),
        "attendu aucun résultat, obtenu {}",
        res.len()
    );
}

#[test]
fn sonder_tous_arret() {
    let arret = Arc::new(AtomicBool::new(true));
    let mut c = cible("192.0.2.1", 80);
    c.timeout = Some(Duration::from_secs(5));
    let cibles = vec![c; 20];
    let debut = Instant::now();
    let res = sonder_tous(&cibles, 2, arret);
    assert!(
        debut.elapsed() < Duration::from_secs(1),
        "avec l'arrêt levé, sonder_tous doit rendre la main vite, a pris {:?}",
        debut.elapsed()
    );
    assert_eq!(
        res.len(),
        cibles.len(),
        "un résultat par cible, même annulée"
    );
    for (i, r) in res.iter().enumerate() {
        assert!(!r.ok, "résultat {i} : attendu KO");
        assert_eq!(r.erreur.as_deref(), Some("annulé"), "résultat {i}");
    }
}

#[test]
fn duree_courte_comme_go() {
    assert_eq!(duree_courte(Duration::ZERO), "0s");
    assert_eq!(duree_courte(Duration::from_micros(1500)), "2ms");
    assert_eq!(duree_courte(Duration::from_micros(1200)), "1ms");
    assert_eq!(duree_courte(Duration::from_millis(504)), "504ms");
    assert_eq!(duree_courte(Duration::from_secs(2)), "2s");
    assert_eq!(duree_courte(Duration::from_millis(1500)), "1.5s");
    assert_eq!(duree_courte(Duration::from_secs(90)), "1m30s");
    assert_eq!(TIMEOUT_DEFAUT, Duration::from_secs(2));
}

#[test]
fn format_tableau_a_l_espace_pres() {
    let res = vec![
        Resultat {
            cible: cible("db1", 5432),
            ok: true,
            duree: Duration::from_micros(1500),
            erreur: None,
        },
        Resultat {
            cible: cible("web1", 443),
            ok: false,
            duree: Duration::from_secs(2),
            erreur: Some("i/o timeout".to_string()),
        },
    ];
    let veut = "CIBLE                  ETAT    DUREE  DETAIL\n".to_string()
        + "db1:5432               OK        2ms  \n"
        + "web1:443               KO         2s  i/o timeout\n";
    assert_eq!(format_tableau(&res), veut);
    assert!(
        format_tableau(&[]).starts_with("CIBLE"),
        "même sans résultat, l'en-tête doit être présent"
    );
}
