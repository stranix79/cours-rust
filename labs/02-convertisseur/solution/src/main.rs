// Labo 02, solution : affiche un tableau de démonstration du convertisseur.
// Lancé par : (cd solution && cargo run)   depuis labs/02-convertisseur
//
// main.rs ne contient aucune logique : il importe les fonctions de la
// bibliothèque (src/lib.rs, nommée `convertisseur` dans Cargo.toml) et
// aligne les résultats en colonnes.

use convertisseur::{celsius, fahrenheit, parse_duree, secondes, taille_lisible};

fn main() {
    // Un tableau de f64 : la taille (5) fait partie du type, le compilateur
    // la déduit du littéral.
    let temperatures = [-40.0, 0.0, 20.0, 37.0, 100.0];
    // {:<10} aligne à gauche sur 10 colonnes, {:>12} à droite sur 12.
    println!("{:<10}{:>12}", "Celsius", "Fahrenheit");
    // `for t in temperatures` parcourt les valeurs ; un f64 est Copy, la
    // boucle travaille sur des copies et le tableau reste utilisable.
    for t in temperatures {
        // {:<10.1} : aligné à gauche, une décimale. Largeur et précision
        // se combinent comme dans le printf du C.
        println!("{:<10.1}{:>12.1}", t, fahrenheit(t));
    }
    // Aller-retour : 98.6 °F redonne 37 °C, aux arrondis du f64 près.
    println!("{:<10}{:>12.1}", "98.6 °F", celsius(98.6));
    println!();

    // u64 partout : une taille de fichier ne tient pas toujours sur 32 bits.
    // L'annotation [u64; 6] donne leur type aux littéraux : sans elle,
    // 3 << 40 serait un i32 et le compilateur refuserait le débordement.
    let tailles: [u64; 6] = [
        512,
        1536,
        10 * 1024 * 1024,
        2_684_354_560,
        3 << 40,
        5000 << 40,
    ];
    println!("{:<16}{:>12}", "Octets", "Lisible");
    for o in tailles {
        println!("{:<16}{:>12}", o, taille_lisible(o));
    }
    println!();

    // Un tableau de &str : des tranches de chaînes littérales.
    let durees = ["1h30m", "45s", "250ms", "2h", "1h1m1s1ms", "10x", ""];
    println!("{:<12}{:>10}", "Durée", "Secondes");
    for d in durees {
        // parse_duree rend un Result : on traite les deux cas. {d:?} affiche
        // la chaîne entre guillemets, pour voir la chaîne vide.
        match parse_duree(d) {
            Ok(duree) => println!("{:<12}{:>10}", format!("{d:?}"), secondes(duree)),
            Err(e) => println!("{:<12}{:>10}  ({e})", format!("{d:?}"), "erreur"),
        }
    }
}
