// Labo 10 : ce que pèsent les types, en octets.
// Lancé par : cargo run   (la section « Tailles »)
//
// size_of donne la taille de la valeur elle-même, celle qui vit sur la pile
// ou dans le champ d'une struct : pas ce qu'elle possède sur le tas. Une
// String de 1 Go fait 24 octets ; ses données sont ailleurs.

//! La taille en octets des types courants, mesurée par `size_of`.

use std::mem::size_of;

/// Le nom et la taille d'une vingtaine de types courants, dans l'ordre
/// d'affichage.
pub fn tailles() -> Vec<(&'static str, usize)> {
    // TODO 10 : complète la liste, une paire (nom, size_of::<Type>()) par
    // type, avec au moins : u64, u128, char, bool, (), &u8, &str, &[u64],
    // String, Vec<u8>, Box<u8>, Rc<u8> (use std::rc::Rc), Option<u8>,
    // Option<u32>, Option<Box<u8>>, Option<String>, Option<char>,
    // Box<dyn Fn()> et [u8; 100]. Le nom doit être écrit exactement comme
    // ici : les tests cherchent "Option<Box<u8>>", "&str"... Avant de lancer,
    // devine chaque taille, puis compare : c'est tout l'intérêt.
    [("u8", size_of::<u8>())].to_vec()
}

/// La taille d'un type, cherchée par son nom dans [`tailles`]. Pratique pour
/// les tests et l'affichage.
pub fn taille(nom: &str) -> Option<usize> {
    tailles()
        .into_iter()
        .find(|(n, _)| *n == nom)
        .map(|(_, t)| t)
}
