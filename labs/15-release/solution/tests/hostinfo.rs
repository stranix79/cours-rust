// Labo 15 : tests de hostinfo. Les fonctions de lecture et de format sont
// testées sur des entrées fixes : les tests passent pareil sur Linux, macOS
// et Windows, ce que la matrice de la CI vérifie.
// Lancé par : cargo test

use hostinfo::{
    Infos, VERSION, collecter, format_duree, format_texte, lire_kern_boottime, lire_proc_uptime,
};

fn exemple() -> Infos {
    Infos {
        machine: "web-01".to_string(),
        systeme: "linux".to_string(),
        architecture: "x86_64".to_string(),
        coeurs: 4,
        version: "1.2.0".to_string(),
        uptime_secondes: Some(4_591_920),
    }
}

#[test]
fn version_vient_de_cargo_toml() {
    // Le test lit la même variable que la bibliothèque : si quelqu'un écrit
    // la version en dur dans le code, il échoue au prochain changement.
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(VERSION, "1.2.0");
}

#[test]
fn proc_uptime() {
    assert_eq!(
        lire_proc_uptime("4591920.53 36581233.10\n"),
        Some(4_591_920)
    );
    assert_eq!(lire_proc_uptime("12 34"), Some(12));
    assert_eq!(lire_proc_uptime(""), None);
    assert_eq!(lire_proc_uptime("pas un nombre"), None);
    assert_eq!(lire_proc_uptime("-3.5 0"), None, "pas de durée négative");
}

#[test]
fn kern_boottime() {
    let sortie = "{ sec = 1788357618, usec = 302303 } Wed Sep  2 16:00:18 2026\n";
    assert_eq!(lire_kern_boottime(sortie, 1_792_949_538), Some(4_591_920));
    assert_eq!(lire_kern_boottime(sortie, 1_788_357_618), Some(0));
    assert_eq!(
        lire_kern_boottime(sortie, 1_000),
        None,
        "démarrage dans le futur"
    );
    assert_eq!(lire_kern_boottime("", 1_000), None);
    assert_eq!(lire_kern_boottime("{ sec = abc, usec = 1 }", 1_000), None);
}

#[test]
fn durees() {
    assert_eq!(format_duree(0), "0 s");
    assert_eq!(format_duree(59), "59 s");
    assert_eq!(format_duree(60), "1 min");
    assert_eq!(format_duree(3_599), "59 min");
    assert_eq!(format_duree(3_600), "1 h 0 min");
    assert_eq!(format_duree(11_520), "3 h 12 min");
    assert_eq!(format_duree(86_400), "1 j 0 h 0 min");
    assert_eq!(format_duree(4_591_920), "53 j 3 h 32 min");
}

#[test]
fn texte_aligne() {
    assert_eq!(
        format_texte(&exemple()),
        "machine      : web-01\n\
         système      : linux\n\
         architecture : x86_64\n\
         cœurs        : 4\n\
         version      : 1.2.0\n\
         uptime       : 53 j 3 h 32 min\n"
    );
    let sans_uptime = Infos {
        uptime_secondes: None,
        ..exemple()
    };
    assert!(format_texte(&sans_uptime).ends_with("uptime       : inconnu\n"));
}

#[test]
fn json_stable() {
    // Le JSON est un contrat pour les scripts qui le lisent : on fige les
    // noms des clés et le null pour une valeur absente.
    let json = serde_json::to_value(Infos {
        uptime_secondes: None,
        ..exemple()
    })
    .expect("JSON");
    assert_eq!(
        json,
        serde_json::json!({
            "machine": "web-01",
            "systeme": "linux",
            "architecture": "x86_64",
            "coeurs": 4,
            "version": "1.2.0",
            "uptime_secondes": null
        })
    );
}

#[test]
fn collecter_la_machine_courante() {
    let infos = collecter();
    assert!(!infos.machine.is_empty());
    assert_eq!(infos.systeme, std::env::consts::OS);
    assert_eq!(infos.architecture, std::env::consts::ARCH);
    assert!(infos.coeurs >= 1);
    assert_eq!(infos.version, VERSION);
    // Linux et macOS savent dire depuis quand ils tournent ; Windows non,
    // et le test l'accepte.
    if cfg!(any(target_os = "linux", target_os = "macos")) {
        assert!(
            infos.uptime_secondes.is_some(),
            "uptime attendu sur ce système"
        );
    }
}
