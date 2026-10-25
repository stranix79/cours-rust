// Labo 02 : la bibliothèque du convertisseur.
// Testé par : cargo test   Lancé par : cargo run (main.rs affiche une démo)
//
// Complète les TODO dans l'ordre. Tout compile dès maintenant : chaque
// fonction à écrire contient un todo!(), qui panique à l'exécution, donc les
// tests échouent tant que tu ne l'as pas remplacé.

// Les paramètres des fonctions pas encore écrites sont inutilisés, ce que
// le compilateur signale. Retire cette ligne quand tous les TODO sont faits.
#![allow(unused_variables)]

use std::time::Duration;

/// Convertit des degrés Celsius en degrés Fahrenheit.
pub fn fahrenheit(celsius: f64) -> f64 {
    // TODO 1 : F = C × 9 / 5 + 32. Écris les constantes en f64 (9.0, pas 9) :
    // Rust ne multiplie pas un f64 par un i32. Pas besoin de `return` : la
    // dernière expression sans point-virgule est la valeur rendue.
    todo!("TODO 1")
}

/// Convertit des degrés Fahrenheit en degrés Celsius.
pub fn celsius(fahrenheit: f64) -> f64 {
    // TODO 2 : C = (F - 32) × 5 / 9.
    todo!("TODO 2")
}

/// Rend une taille en octets lisible par un humain, en base 1024 :
/// `512 o`, `1.5 Ko`, `10.0 Mo`. Au-delà du To, on reste en To.
pub fn taille_lisible(octets: u64) -> String {
    // TODO 3 : sous 1024, format!("{octets} o"). Sinon, convertis en f64
    // (`octets as f64 / 1024.0`), puis divise par 1024 tant que la valeur
    // dépasse 1024 et qu'il reste une unité plus grande dans
    // ["Ko", "Mo", "Go", "To"]. Affiche avec une décimale : {valeur:.1}.
    todo!("TODO 3")
}

/// Analyse une durée écrite comme `1h30m`, `45s`, `250ms` ou `2h` :
/// une suite de nombres entiers suivis chacun d'une unité (`h`, `m`, `s`,
/// `ms`). Renvoie une erreur, avec le texte fautif, si la chaîne est vide,
/// si une unité est inconnue ou manquante, ou si un nombre manque.
pub fn parse_duree(texte: &str) -> Result<Duration, String> {
    // TODO 4 : la std n'a pas d'analyseur de durées, on l'écrit.
    // - Chaîne vide : return Err(format!("durée vide : {texte:?}")).
    // - Parcours `for c in texte.chars()` avec deux String : `nombre` (les
    //   chiffres, c.is_ascii_digit()) et `unite` (les autres caractères).
    // - Quand un chiffre arrive alors que `unite` n'est pas vide, la
    //   composante précédente est complète : convertis-la, ajoute-la au
    //   total, et vide les deux String avec .clear(). Fais de même après
    //   la boucle pour la dernière composante.
    // - Convertir une composante : match unite { "h" => 3_600_000, "m" =>
    //   60_000, "s" => 1_000, "ms" => 1, _ => erreur } donne le facteur en
    //   millisecondes ; nombre.parse::<u64>() rend un Result (match dessus) ;
    //   un nombre vide ou une unité vide sont des erreurs.
    // - Chaque message d'erreur contient le texte complet : {texte:?}.
    // - Termine par Ok(Duration::from_millis(total_ms)).
    // Une fonction auxiliaire `fn composante(nombre: &str, unite: &str,
    // texte: &str) -> Result<u64, String>` rend le tout plus lisible.
    todo!("TODO 4")
}

/// Le nombre de secondes entières d'une durée, tronqué : 1,9 s donne 1.
pub fn secondes(d: Duration) -> u64 {
    // TODO 5 : une méthode de Duration fait exactement ça. Cherche-la avec
    // `cargo doc --open` ou sur https://doc.rust-lang.org/std/time/struct.Duration.html
    todo!("TODO 5")
}
