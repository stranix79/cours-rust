// Tests du labo 02. Ils appellent la bibliothèque `convertisseur` comme le
// ferait n'importe quel programme : seules les fonctions `pub` sont visibles.
// Lancés par : cargo test   (un seul : cargo test taille)

use convertisseur::{celsius, fahrenheit, parse_duree, secondes, taille_lisible};
use std::time::Duration;

// Deux f64 ne se comparent pas avec == après un calcul : 0.1 + 0.2 != 0.3.
// On tolère une petite différence.
fn proche(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn fahrenheit_valeurs_connues() {
    assert!(proche(fahrenheit(0.0), 32.0), "0 °C = 32 °F");
    assert!(proche(fahrenheit(100.0), 212.0), "100 °C = 212 °F");
    assert!(proche(fahrenheit(-40.0), -40.0), "-40 °C = -40 °F");
    assert!(proche(fahrenheit(37.0), 98.6), "37 °C = 98.6 °F");
}

#[test]
fn celsius_valeurs_connues() {
    assert!(proche(celsius(32.0), 0.0), "32 °F = 0 °C");
    assert!(proche(celsius(212.0), 100.0), "212 °F = 100 °C");
    assert!(proche(celsius(-40.0), -40.0), "-40 °F = -40 °C");
}

#[test]
fn celsius_aller_retour() {
    for t in [-273.15, -12.5, 0.0, 21.3, 1000.0] {
        assert!(proche(celsius(fahrenheit(t)), t), "aller-retour pour {t}");
    }
}

#[test]
fn taille_sous_un_ko() {
    assert_eq!(taille_lisible(0), "0 o");
    assert_eq!(taille_lisible(512), "512 o");
    assert_eq!(taille_lisible(1023), "1023 o");
}

#[test]
fn taille_unites() {
    assert_eq!(taille_lisible(1024), "1.0 Ko");
    assert_eq!(taille_lisible(1536), "1.5 Ko");
    assert_eq!(taille_lisible(10 * 1024 * 1024), "10.0 Mo");
    assert_eq!(taille_lisible(2_684_354_560), "2.5 Go");
    assert_eq!(taille_lisible(3 << 40), "3.0 To");
}

#[test]
fn taille_reste_en_to() {
    assert_eq!(taille_lisible(5000 << 40), "5000.0 To");
    assert_eq!(taille_lisible(u64::MAX), "16777216.0 To");
}

#[test]
fn duree_simple() {
    assert_eq!(parse_duree("45s"), Ok(Duration::from_secs(45)));
    assert_eq!(parse_duree("250ms"), Ok(Duration::from_millis(250)));
    assert_eq!(parse_duree("2h"), Ok(Duration::from_secs(7200)));
    assert_eq!(parse_duree("15m"), Ok(Duration::from_secs(900)));
}

#[test]
fn duree_composee() {
    assert_eq!(parse_duree("1h30m"), Ok(Duration::from_secs(5400)));
    assert_eq!(
        parse_duree("1h1m1s1ms"),
        Ok(Duration::from_millis(3_661_001))
    );
    assert_eq!(parse_duree("1m30s500ms"), Ok(Duration::from_millis(90_500)));
}

#[test]
fn duree_vide() {
    assert!(parse_duree("").is_err(), "une chaîne vide est une erreur");
}

#[test]
fn duree_unite_inconnue() {
    let erreur = parse_duree("10x").unwrap_err();
    assert!(
        erreur.contains("10x"),
        "le message doit citer le texte : {erreur}"
    );
    let erreur = parse_duree("1h30j").unwrap_err();
    assert!(
        erreur.contains("1h30j"),
        "le message doit citer le texte : {erreur}"
    );
}

#[test]
fn duree_mal_formee() {
    assert!(
        parse_duree("45").is_err(),
        "un nombre sans unité est une erreur"
    );
    assert!(
        parse_duree("h").is_err(),
        "une unité sans nombre est une erreur"
    );
    assert!(parse_duree("1.5h").is_err(), "pas de décimales");
    assert!(parse_duree(" 45s").is_err(), "pas d'espace");
}

#[test]
fn secondes_tronque() {
    assert_eq!(secondes(Duration::from_secs(5400)), 5400);
    assert_eq!(secondes(Duration::from_millis(1999)), 1);
    assert_eq!(secondes(Duration::from_millis(999)), 0);
    assert_eq!(secondes(parse_duree("1h30m").unwrap()), 5400);
}
