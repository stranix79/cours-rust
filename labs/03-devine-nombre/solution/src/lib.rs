// Labo 03, solution : la logique du jeu du nombre à deviner.
// Testé par : (cd solution && cargo test)   Lancé par : (cd solution && cargo run)
//
// Tout ce qui est testable est ici. Le hasard et le clavier restent dans
// main.rs : une fonction qui tire un nombre au sort ou lit l'entrée standard
// ne se teste pas facilement, alors on lui passe le secret et les essais en
// paramètres. C'est le même découpage qu'en Go : main mince, logique pure.

/// Ce que le jeu répond à un essai.
// derive génère du code à la compilation : Debug pour l'afficher avec {:?}
// (assert_eq! en a besoin pour son message), PartialEq et Eq pour le
// comparer avec ==, Clone et Copy pour le copier comme un entier au lieu de
// le déplacer (chapitre 4). Les enums ont leur chapitre, le 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    TropPetit,
    TropGrand,
    Trouve,
}

/// Le bilan d'une partie rejouée par `partie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resultat {
    /// Le secret a-t-il été trouvé ?
    pub trouve: bool,
    /// Nombre d'essais valides joués (le gagnant compris).
    pub tentatives: u32,
    /// Nombre de lignes ignorées parce qu'elles n'étaient pas un essai valide.
    pub invalides: u32,
}

/// Compare un essai au secret.
pub fn evaluer(secret: u32, essai: u32) -> Verdict {
    // Un match sur une comparaison : cmp() rend un std::cmp::Ordering, un
    // enum à trois valeurs (Less, Greater, Equal). Le compilateur vérifie
    // que les trois sont traitées ; en oublier une ne compile pas. C'est
    // plus sûr qu'une chaîne de if / else if où l'on peut oublier un cas.
    match essai.cmp(&secret) {
        std::cmp::Ordering::Less => Verdict::TropPetit,
        std::cmp::Ordering::Greater => Verdict::TropGrand,
        std::cmp::Ordering::Equal => Verdict::Trouve,
    }
}

/// Lit un essai tapé par le joueur : un entier de 1 à 100, espaces et
/// retour à la ligne tolérés. Rend None pour tout le reste.
pub fn lire_essai(texte: &str) -> Option<u32> {
    // trim() enlève les blancs des deux côtés, y compris le "\n" que
    // read_line laisse à la fin de la ligne. Il rend une &str : une vue sur
    // le texte d'origine, sans copie.
    // parse::<u32>() rend un Result<u32, ParseIntError> : Err pour "abc",
    // "-5", "12.5" ou un nombre trop grand pour un u32.
    match texte.trim().parse::<u32>() {
        // Un seul bras fait les deux vérifications : le motif Ok(...) exige
        // que l'analyse ait réussi, et `n @ 1..=100` exige que la valeur
        // soit dans la plage (le 100 compris) tout en la liant au nom n.
        Ok(n @ 1..=100) => Some(n),
        // Tout le reste : Ok(0), Ok(101), et toutes les Err. Le message
        // d'erreur de parse ne nous intéresse pas, on le jette.
        _ => None,
    }
}

/// Rejoue une partie à partir d'une liste d'essais, sans clavier ni hasard.
/// Les lignes invalides sont comptées à part et ne coûtent pas de
/// tentative. La partie s'arrête au premier essai gagnant.
pub fn partie(secret: u32, essais: &[&str]) -> Resultat {
    // `essais: &[&str]` : une tranche empruntée de &str. La fonction lit la
    // liste sans la prendre à l'appelant, et accepte aussi bien un tableau
    // (&["50", "75"]) qu'un Vec<&str> (&v). Chapitre 4 pour les tranches.
    // `mut` : sans lui, le compilateur refuserait de modifier les champs.
    let mut resultat = Resultat {
        trouve: false,
        tentatives: 0,
        invalides: 0,
    };
    // `for ligne in essais` sur une &[&str] donne des &&str : une référence
    // vers chaque élément. On s'en sert comme d'une &str, le compilateur
    // déréférence tout seul à l'appel de lire_essai.
    for ligne in essais {
        // let else encore : si lire_essai rend None, on compte la ligne
        // invalide et `continue` passe directement à la suivante. Le bloc
        // else doit sortir (ici de ce tour de boucle), sinon ça ne compile pas.
        let Some(essai) = lire_essai(ligne) else {
            resultat.invalides += 1;
            continue;
        };
        resultat.tentatives += 1;
        if evaluer(secret, essai) == Verdict::Trouve {
            resultat.trouve = true;
            // break sort de la boucle : les essais suivants sont ignorés.
            break;
        }
    }
    resultat
}

/// Le message affiché au joueur après un essai. `tentatives` ne sert que
/// pour la victoire, et accorde « tentative » au singulier pour 1.
pub fn message(verdict: Verdict, tentatives: u32) -> String {
    // Le match est une expression : sa valeur est la String renvoyée.
    // to_string() transforme une &str littérale en String possédée,
    // format! en fabrique une directement.
    match verdict {
        Verdict::TropPetit => "C'est plus grand.".to_string(),
        Verdict::TropGrand => "C'est plus petit.".to_string(),
        // Une garde (`if`) affine le motif : ce bras ne prend que le cas
        // tentatives == 1. L'ordre compte, le premier bras qui correspond
        // gagne.
        Verdict::Trouve if tentatives == 1 => "Trouvé du premier coup !".to_string(),
        Verdict::Trouve => format!("Trouvé en {tentatives} tentatives."),
    }
}
