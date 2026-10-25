// Labo 08 : les tests du lecteur de configuration. Identiques dans le
// squelette et dans la solution : `use robuste::...` désigne la bibliothèque
// du paquet, et les fichiers d'exemple sont lus dans le dossier du paquet
// (CARGO_MANIFEST_DIR), où ils existent en deux exemplaires.
// Lancé par : cargo test (ton code) ou (cd solution && cargo test).

use std::error::Error;
use std::io;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::time::Duration;

use robuste::*;

// Le chemin d'un fichier du dossier du paquet. env! est lu à la compilation :
// c'est le dossier qui contient le Cargo.toml du paquet testé.
fn fichier(nom: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(nom)
}

// Une erreur attendue : panique (le test échoue) si parser réussit.
fn erreur(texte: &str) -> ErreurConfig {
    match parser(texte) {
        Ok(config) => panic!("erreur attendue pour {texte:?}, obtenu {config:?}"),
        Err(e) => e,
    }
}

#[test]
fn exemple_conf_complet() {
    let config = charger(&fichier("exemple.conf")).expect("exemple.conf est valide");
    assert_eq!(
        config,
        Config {
            port: 5432,
            timeout: Duration::from_secs(3),
            hotes: vec![
                "db-01".to_string(),
                "db-02".to_string(),
                "cache-01".to_string()
            ],
            verbeux: true,
        }
    );
}

#[test]
fn valeurs_par_defaut() {
    let config = parser("port = 80\nhotes = web-01\n").expect("configuration minimale");
    assert_eq!(config.timeout, Duration::from_secs(5), "timeout par défaut");
    assert!(!config.verbeux, "verbeux par défaut");
    assert_eq!(config.hotes, ["web-01"]);
}

#[test]
fn espaces_commentaires_et_repetitions() {
    let texte = "\n   # commentaire indenté\n\tport=22\t\nhotes=a,,b,\nport = 2222\n\n";
    let config = parser(texte).expect("valide");
    assert_eq!(config.port, 2222, "une clé répétée : la dernière gagne");
    assert_eq!(config.hotes, ["a", "b"], "les morceaux vides sont ignorés");
}

#[test]
fn invalide_conf_signale_la_ligne_3() {
    let e = charger(&fichier("invalide.conf")).unwrap_err();
    assert!(
        matches!(&e, ErreurConfig::ValeurInvalide { numero: 3, cle, .. } if cle == "port"),
        "obtenu {e:?}"
    );
    assert_eq!(e.to_string(), "ligne 3 : valeur invalide pour « port »");
    let cause = e.source().expect("une valeur invalide a une cause");
    assert_eq!(cause.to_string(), "invalid digit found in string");
    assert!(
        cause.downcast_ref::<ParseIntError>().is_some(),
        "la cause d'origine est gardée telle quelle"
    );
}

#[test]
fn fichier_absent() {
    let chemin = fichier("absent.conf");
    let e = charger(&chemin).unwrap_err();
    match &e {
        ErreurConfig::Io { chemin: c, source } => {
            assert_eq!(c, &chemin);
            assert_eq!(source.kind(), io::ErrorKind::NotFound);
        }
        autre => panic!("Io attendu, obtenu {autre:?}"),
    }
    assert_eq!(e.to_string(), format!("lecture de {}", chemin.display()));
    assert!(e.source().is_some(), "l'io::Error est la source");
}

#[test]
fn ligne_sans_egal() {
    let e = erreur("hotes = a\n  port 5432  \n");
    assert!(
        matches!(&e, ErreurConfig::LigneInvalide { numero: 2, contenu } if contenu == "port 5432"),
        "obtenu {e:?}"
    );
    assert_eq!(
        e.to_string(),
        "ligne 2 : « port 5432 » n'est pas de la forme cle = valeur"
    );
    assert!(e.source().is_none());
}

#[test]
fn cle_inconnue() {
    let e = erreur("port = 1\n# ok\nprot = 2\n");
    assert!(
        matches!(&e, ErreurConfig::CleInconnue { numero: 3, cle } if cle == "prot"),
        "obtenu {e:?}"
    );
    assert_eq!(e.to_string(), "ligne 3 : clé inconnue « prot »");
}

#[test]
fn cles_manquantes() {
    let e = erreur("hotes = a\n");
    assert!(
        matches!(e, ErreurConfig::CleManquante("port")),
        "obtenu {e:?}"
    );
    assert_eq!(e.to_string(), "clé obligatoire absente : « port »");

    let e = erreur("# rien\nport = 80\n");
    assert!(
        matches!(e, ErreurConfig::CleManquante("hotes")),
        "obtenu {e:?}"
    );
    assert!(matches!(erreur(""), ErreurConfig::CleManquante(_)));
}

#[test]
fn valeurs_invalides_et_leurs_causes() {
    // (texte, numéro de ligne, clé, message de la cause)
    let cas = [
        (
            "port = 70000\nhotes = a",
            1,
            "port",
            "number too large to fit in target type",
        ),
        ("hotes = a\nport = 0", 2, "port", "le port 0 est réservé"),
        (
            "port = 1\nhotes = a\ntimeout = 3s",
            3,
            "timeout",
            "invalid digit found in string",
        ),
        (
            "port = 1\nhotes = a\nverbeux = oui",
            3,
            "verbeux",
            "provided string was not `true` or `false`",
        ),
        (
            "port = 1\nhotes = , ,",
            2,
            "hotes",
            "la liste d'hôtes est vide",
        ),
    ];
    for (texte, ligne, nom, message) in cas {
        let e = erreur(texte);
        match &e {
            ErreurConfig::ValeurInvalide {
                numero,
                cle,
                source,
            } => {
                assert_eq!((*numero, cle.as_str()), (ligne, nom), "pour {texte:?}");
                assert_eq!(source.to_string(), message, "pour {texte:?}");
            }
            autre => panic!("ValeurInvalide attendu pour {texte:?}, obtenu {autre:?}"),
        }
    }
}

#[test]
fn erreur_compatible_anyhow() {
    // Send + Sync + 'static : ce qu'anyhow exige pour accepter l'erreur.
    fn accepte<E: Error + Send + Sync + 'static>(_: &E) {}
    let e = charger(&fichier("invalide.conf")).unwrap_err();
    accepte(&e);

    // La chaîne complète, telle que main l'affiche avec {:#}.
    let chaine = format!("{:#}", anyhow::Error::new(e).context("configuration"));
    assert_eq!(
        chaine,
        "configuration: ligne 3 : valeur invalide pour « port »: invalid digit found in string"
    );
}
