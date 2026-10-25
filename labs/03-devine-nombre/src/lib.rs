// Labo 03 : la logique du jeu du nombre à deviner.
// Testé par : cargo test   Lancé par : cargo run (main.rs fait jouer au clavier)
//
// Complète les TODO dans l'ordre. Tout compile dès maintenant : chaque
// fonction à écrire contient un todo!(), qui panique à l'exécution.
// Le hasard et le clavier restent dans main.rs : ici, le secret et les
// essais arrivent en paramètres, ce qui rend tout testable.

// Les paramètres des fonctions pas encore écrites sont inutilisés, ce que
// le compilateur signale. Retire cette ligne quand tous les TODO sont faits.
#![allow(unused_variables)]

/// Ce que le jeu répond à un essai. Les derive permettent de l'afficher
/// ({:?}), de le comparer (==) et de le copier comme un entier.
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
    // TODO 1 : un if / else if / else suffit, mais essaie le match sur
    // essai.cmp(&secret), qui rend un std::cmp::Ordering (Less, Greater,
    // Equal) : le compilateur vérifiera que tu traites les trois cas.
    todo!("TODO 1")
}

/// Lit un essai tapé par le joueur : un entier de 1 à 100, espaces et
/// retour à la ligne tolérés. Rend None pour tout le reste.
pub fn lire_essai(texte: &str) -> Option<u32> {
    // TODO 2 : texte.trim() enlève les blancs (et le "\n" de read_line),
    // .parse::<u32>() rend un Result. Un match sur ce Result avec le motif
    // Ok(n @ 1..=100) => Some(n) fait l'analyse et la vérification de plage
    // en un seul bras ; `_ => None` pour le reste.
    todo!("TODO 2")
}

/// Rejoue une partie à partir d'une liste d'essais, sans clavier ni hasard.
/// Les lignes invalides sont comptées à part et ne coûtent pas de
/// tentative. La partie s'arrête au premier essai gagnant.
pub fn partie(secret: u32, essais: &[&str]) -> Resultat {
    // TODO 3 : un Resultat mutable à zéro, puis `for ligne in essais`.
    // Pour chaque ligne : lire_essai ; si None, invalides += 1 et continue
    // (let Some(essai) = lire_essai(ligne) else { ...; continue; }) ;
    // sinon tentatives += 1, et si evaluer rend Verdict::Trouve, note-le et
    // sors avec break. Rends le Resultat (dernière expression, sans `;`).
    todo!("TODO 3")
}

/// Le message affiché au joueur après un essai. `tentatives` ne sert que
/// pour la victoire, et accorde « tentative » au singulier pour 1.
pub fn message(verdict: Verdict, tentatives: u32) -> String {
    // TODO 4 : un match sur verdict, une expression qui rend une String.
    // TropPetit : "C'est plus grand." ; TropGrand : "C'est plus petit." ;
    // Trouve avec 1 tentative : "Trouvé du premier coup !" (une garde :
    // `Verdict::Trouve if tentatives == 1 =>`) ; sinon
    // format!("Trouvé en {tentatives} tentatives."). Pour une &str
    // littérale, .to_string() en fait une String.
    todo!("TODO 4")
}
