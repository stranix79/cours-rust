// Labo 09 : les tests d'intégration de semver_maison. Identiques dans le
// squelette et dans la solution.
//
// Chaque fichier de tests/ est compilé comme une crate à part, qui importe
// la bibliothèque comme le ferait n'importe quel utilisateur : il ne voit que
// l'API publique (ce que lib.rs exporte), jamais les fonctions privées.

use semver_maison::{Contrainte, ErreurContrainte, ErreurVersion, Version};

/// Lit une version ou arrête le test avec un message qui dit laquelle.
fn v(s: &str) -> Version {
    Version::parse(s).unwrap_or_else(|e| panic!("Version::parse({s:?}) : {e}"))
}

#[test]
fn parse_versions_valides() {
    let cas = [
        ("1.2.3", (1, 2, 3), None, None),
        ("v1.2.3", (1, 2, 3), None, None),
        (" 1.2.3\n", (1, 2, 3), None, None),
        ("0.0.0", (0, 0, 0), None, None),
        ("10.200.3000", (10, 200, 3000), None, None),
        ("1.2.3-beta.1", (1, 2, 3), Some("beta.1"), None),
        ("1.2.3+build.5", (1, 2, 3), None, Some("build.5")),
        (
            "1.2.3-beta.1+build.5",
            (1, 2, 3),
            Some("beta.1"),
            Some("build.5"),
        ),
        ("1.0.0-alpha-1", (1, 0, 0), Some("alpha-1"), None),
    ];
    for (texte, triplet, pre, build) in cas {
        let x = v(texte);
        assert_eq!(x.triplet(), triplet, "{texte:?}");
        assert_eq!(x.prerelease.as_deref(), pre, "{texte:?} : préversion");
        assert_eq!(x.build.as_deref(), build, "{texte:?} : build");
    }
}

#[test]
fn parse_versions_invalides() {
    let cas = [
        ("", ErreurVersion::Vide),
        ("1.2", ErreurVersion::Forme("1.2".into())),
        ("1.2.3.4", ErreurVersion::Forme("1.2.3.4".into())),
        ("a.b.c", ErreurVersion::Nombre("a".into())),
        ("1.02.3", ErreurVersion::Nombre("02".into())),
        ("1..3", ErreurVersion::Nombre("".into())),
        ("1.2.3-", ErreurVersion::Identifiant("".into())),
        ("1.2.3-beta..1", ErreurVersion::Identifiant("".into())),
        ("1.2.3-01", ErreurVersion::Identifiant("01".into())),
        ("1.2.3+", ErreurVersion::Identifiant("".into())),
    ];
    for (texte, erreur) in cas {
        assert_eq!(Version::parse(texte), Err(erreur), "{texte:?}");
    }
}

#[test]
fn affichage_aller_retour() {
    for texte in [
        "1.2.3",
        "0.1.0-rc.1",
        "1.2.3-beta.1+build.5",
        "2.0.0+sha.abc",
    ] {
        assert_eq!(v(texte).to_string(), texte);
    }
    // FromStr : la même lecture par .parse::<Version>().
    let x: Version = "3.4.5".parse().unwrap();
    assert_eq!(x, v("3.4.5"));
}

#[test]
fn precedence_semver() {
    // L'exemple de la spécification semver, dans l'ordre croissant.
    let ordre = [
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
        "1.2.0",
        "1.10.0",
        "2.0.0",
    ];
    for paire in ordre.windows(2) {
        assert!(v(paire[0]) < v(paire[1]), "{} < {}", paire[0], paire[1]);
    }
}

#[test]
fn le_build_est_ignore() {
    assert_eq!(v("1.0.0+a"), v("1.0.0+b"));
    assert_eq!(v("1.0.0+a").cmp(&v("1.0.0")), std::cmp::Ordering::Equal);
}

#[test]
fn trier_un_vec() {
    let mut liste: Vec<Version> = ["1.10.0", "1.2.3", "1.2.3-rc.1", "0.9.0"]
        .iter()
        .map(|s| v(s))
        .collect();
    liste.sort();
    let textes: Vec<String> = liste.iter().map(|x| x.to_string()).collect();
    assert_eq!(textes, ["0.9.0", "1.2.3-rc.1", "1.2.3", "1.10.0"]);
    assert_eq!(liste.iter().max().unwrap().to_string(), "1.10.0");
}

#[test]
fn contraintes() {
    let cas = [
        ("^1.2", "1.2.0", true),
        ("^1.2", "1.9.3", true),
        ("^1.2", "2.0.0", false),
        ("^1.2", "1.1.9", false),
        ("^1.2.3", "1.2.2", false),
        ("1.2", "1.5.0", true), // sans opérateur : un caret, comme Cargo.toml
        ("^0.2.3", "0.2.9", true),
        ("^0.2.3", "0.3.0", false), // avant 1.0, le mineur casse
        ("^0.0.3", "0.0.4", false),
        ("~1.2.3", "1.2.9", true),
        ("~1.2.3", "1.3.0", false),
        ("~1.2", "1.2.0", true),
        ("~1", "1.9.0", true),
        ("~1", "2.0.0", false),
        ("=1.2.3", "1.2.3", true),
        ("=1.2.3", "1.2.4", false),
        ("=1.2", "1.2.7", true),
        (">=1.0, <2.0", "1.5.0", true),
        (">=1.0, <2.0", "2.0.0", false),
        (">=1.0, <2.0", "0.9.9", false),
        (" >=1.0 ,<2.0 ", "1.0.0", true),
        (">1.2", "1.2.9", false),
        (">1.2", "1.3.0", true),
        ("<=1.2", "1.2.9", true),
        ("<=1.2", "1.3.0", false),
        ("*", "0.0.1", true),
        ("*", "99.0.0", true),
    ];
    for (contrainte, version, attendu) in cas {
        let c = Contrainte::parse(contrainte)
            .unwrap_or_else(|e| panic!("Contrainte::parse({contrainte:?}) : {e}"));
        assert_eq!(
            c.satisfait(&v(version)),
            attendu,
            "{version} satisfait {contrainte:?} ?"
        );
    }
}

#[test]
fn contraintes_et_preversions() {
    let c = Contrainte::parse("^1.2").unwrap();
    assert!(
        !c.satisfait(&v("1.3.0-alpha")),
        "pas de préversion par surprise"
    );
    let c = Contrainte::parse(">=1.2.3-beta").unwrap();
    assert!(c.satisfait(&v("1.2.3-rc.1")), "même triplet : permise");
    assert!(c.satisfait(&v("1.2.3")));
    assert!(!c.satisfait(&v("1.2.4-rc.1")), "autre triplet : refusée");
    assert!(!c.satisfait(&v("1.2.3-alpha")), "alpha < beta");
}

#[test]
fn contraintes_invalides() {
    assert_eq!(Contrainte::parse("").unwrap_err(), ErreurContrainte::Vide);
    assert_eq!(
        Contrainte::parse("1.2,").unwrap_err(),
        ErreurContrainte::Vide
    );
    assert!(matches!(
        Contrainte::parse("^abc"),
        Err(ErreurContrainte::Partielle(_))
    ));
    assert!(matches!(
        Contrainte::parse(">=1.02.3"),
        Err(ErreurContrainte::Version(ErreurVersion::Nombre(_)))
    ));
}

#[test]
fn les_erreurs_sont_des_erreurs() {
    // Display et std::error::Error : utilisables avec `?` dans un main qui
    // rend Result<(), Box<dyn Error>>.
    let e: Box<dyn std::error::Error> = Box::new(Version::parse("1.2").unwrap_err());
    assert!(e.to_string().contains("MAJEUR.MINEUR.CORRECTIF"), "{e}");
    let e = Contrainte::parse(">=1.02.3").unwrap_err();
    assert!(
        std::error::Error::source(&e).is_some(),
        "la cause est gardée"
    );
}
