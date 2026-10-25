// Labo 02, solution : la bibliothèque du convertisseur.
// Lancé par : (cd solution && cargo run)   Testé par : (cd solution && cargo test)
//
// Toute la logique est ici, dans src/lib.rs, et pas dans main.rs : les tests
// de tests/convertisseur.rs appellent ces fonctions comme le ferait n'importe
// quel autre programme. main.rs ne fait qu'afficher.

use std::time::Duration; // le type « durée » de la std : secondes + nanosecondes

/// Convertit des degrés Celsius en degrés Fahrenheit.
pub fn fahrenheit(celsius: f64) -> f64 {
    // Que des f64 : 9.0 et pas 9, sinon c'est un i32 et Rust refuse de
    // multiplier un f64 par un i32 (aucune conversion implicite).
    celsius * 9.0 / 5.0 + 32.0
}

/// Convertit des degrés Fahrenheit en degrés Celsius.
pub fn celsius(fahrenheit: f64) -> f64 {
    // Pas de `return` : la dernière expression du bloc, sans point-virgule,
    // est la valeur renvoyée.
    (fahrenheit - 32.0) * 5.0 / 9.0
}

// Les unités au-delà de l'octet. Un tableau de taille fixe de &str : les
// chaînes littérales vivent dans le binaire, d'où le type &'static str
// (chapitre 4). `const` parce que rien ne change jamais à l'exécution.
const UNITES: [&str; 4] = ["Ko", "Mo", "Go", "To"];

/// Rend une taille en octets lisible par un humain, en base 1024 :
/// `512 o`, `1.5 Ko`, `10.0 Mo`. Au-delà du To, on reste en To.
pub fn taille_lisible(octets: u64) -> String {
    // Sous 1024, un entier exact, sans décimale : "512 o".
    if octets < 1024 {
        // format! fabrique une String, comme println! mais sans afficher.
        // {octets} : l'argument en ligne, la variable est capturée par nom.
        return format!("{octets} o");
    }
    // `as f64` : la conversion est explicite. Elle est sans perte jusqu'à
    // 2^53 octets (8 Po), largement assez pour une taille de fichier.
    let mut valeur = octets as f64 / 1024.0;
    // usize : le type des indices. On part de "Ko".
    let mut i = 0;
    // On divise tant qu'on dépasse 1024 ET qu'il reste une unité plus grande.
    // La seconde condition fait rester en To au-delà : 5000 To, pas 4.9 Po.
    while valeur >= 1024.0 && i < UNITES.len() - 1 {
        valeur /= 1024.0;
        i += 1;
    }
    // {valeur:.1} : une décimale, arrondie. UNITES[i] est vérifié à
    // l'exécution (panique hors bornes), mais la boucle garantit i <= 3.
    format!("{valeur:.1} {}", UNITES[i])
}

/// Analyse une durée écrite comme `1h30m`, `45s`, `250ms` ou `2h` :
/// une suite de nombres entiers suivis chacun d'une unité (`h`, `m`, `s`,
/// `ms`). Renvoie une erreur, avec le texte fautif, si la chaîne est vide,
/// si une unité est inconnue ou manquante, ou si un nombre manque.
pub fn parse_duree(texte: &str) -> Result<Duration, String> {
    // Chaîne vide : rien à analyser. {texte:?} affiche la chaîne entre
    // guillemets, ce qui rend visible même une chaîne vide ou des espaces.
    if texte.is_empty() {
        return Err(format!("durée vide : {texte:?}"));
    }
    // On compte tout en millisecondes dans un u64 : c'est la plus petite
    // unité acceptée, et un entier ne connaît pas les erreurs d'arrondi.
    let mut total_ms: u64 = 0;
    // Deux tampons : les chiffres du nombre en cours, puis les lettres de
    // son unité. String::new() ne fait aucune allocation tant qu'il est vide.
    let mut nombre = String::new();
    let mut unite = String::new();

    // chars() parcourt la chaîne caractère par caractère (des char de 4
    // octets), pas octet par octet : un "é" arrive entier.
    for c in texte.chars() {
        if c.is_ascii_digit() {
            // Un chiffre après des lettres : la composante précédente est
            // complète ("1h" dans "1h30m"), on l'ajoute au total.
            if !unite.is_empty() {
                // `?` : si composante renvoie Err, parse_duree renvoie
                // immédiatement cette Err ; sinon on récupère la valeur.
                // C'est un match en un caractère, détaillé au chapitre 8.
                total_ms = ajouter(total_ms, composante(&nombre, &unite, texte)?, texte)?;
                // clear() vide la String sans rendre sa mémoire : on la
                // réutilise pour la composante suivante.
                nombre.clear();
                unite.clear();
            }
            nombre.push(c);
        } else {
            unite.push(c);
        }
    }
    // Fin du texte : la dernière composante n'a été suivie d'aucun chiffre,
    // elle est encore dans les tampons. composante() refusera "45" sans
    // unité, puisque `unite` est vide.
    total_ms = ajouter(total_ms, composante(&nombre, &unite, texte)?, texte)?;
    Ok(Duration::from_millis(total_ms))
}

// Une composante "30" + "m" convertie en millisecondes. Les trois paramètres
// sont des &str : on lit les tampons sans les prendre à l'appelant, qui
// continue à s'en servir (chapitre 4). `texte` ne sert qu'au message d'erreur.
// Pas de `pub` : c'est un détail interne, invisible pour les tests.
fn composante(nombre: &str, unite: &str, texte: &str) -> Result<u64, String> {
    // match sur une &str : chaque bras compare la chaîne entière, et `_`
    // attrape tout le reste. Le compilateur exige ce `_` : un match doit
    // couvrir tous les cas (chapitre 3).
    let facteur: u64 = match unite {
        "h" => 3_600_000, // le _ sépare les milliers, il est ignoré
        "m" => 60_000,
        "s" => 1_000,
        "ms" => 1,
        "" => return Err(format!("unité manquante après {nombre} dans {texte:?}")),
        _ => return Err(format!("unité inconnue {unite:?} dans {texte:?}")),
    };
    if nombre.is_empty() {
        return Err(format!("nombre manquant avant {unite:?} dans {texte:?}"));
    }
    // parse::<u64>() rend un Result : Err si le texte n'est pas un nombre
    // ou s'il dépasse u64::MAX. Ici nombre ne contient que des chiffres,
    // donc seul le dépassement peut arriver.
    let n = match nombre.parse::<u64>() {
        Ok(n) => n,
        Err(_) => return Err(format!("nombre trop grand {nombre} dans {texte:?}")),
    };
    // checked_mul rend None en cas de débordement au lieu de paniquer (en
    // debug) ou d'enrouler en silence (en release). ok_or transforme ce
    // None en Err avec notre message.
    n.checked_mul(facteur)
        .ok_or(format!("durée trop grande dans {texte:?}"))
}

// L'addition du total, protégée contre le débordement de la même façon.
fn ajouter(total: u64, ms: u64, texte: &str) -> Result<u64, String> {
    total
        .checked_add(ms)
        .ok_or(format!("durée trop grande dans {texte:?}"))
}

/// Le nombre de secondes entières d'une durée, tronqué : 1,9 s donne 1.
pub fn secondes(d: Duration) -> u64 {
    // as_secs() ignore la partie fractionnaire : c'est exactement la
    // troncature demandée, sans passer par un flottant.
    d.as_secs()
}
