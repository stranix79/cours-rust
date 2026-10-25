// Labo 10, solution : ce que pèsent les types, en octets.
// Lancé par : (cd solution && cargo run)   la section « Tailles »
//
// size_of donne la taille de la valeur elle-même, celle qui vit sur la pile
// ou dans le champ d'une struct : pas ce qu'elle possède sur le tas. Une
// String de 1 Go fait 24 octets ; ses données sont ailleurs.

//! La taille en octets des types courants, mesurée par `size_of`.

use std::mem::size_of;
use std::rc::Rc;

/// Le nom et la taille d'une vingtaine de types courants, dans l'ordre
/// d'affichage.
pub fn tailles() -> Vec<(&'static str, usize)> {
    // Un tableau de paires, puis to_vec : les noms sont des &'static str,
    // des littéraux gravés dans le binaire, qui vivent tout le programme.
    [
        ("u8", size_of::<u8>()),
        ("u64", size_of::<u64>()),
        ("u128", size_of::<u128>()),
        ("char", size_of::<char>()), // un point de code Unicode : 4 octets
        ("bool", size_of::<bool>()), // un octet, pas un bit
        ("()", size_of::<()>()),     // rien du tout
        ("&u8", size_of::<&u8>()),   // un pointeur
        ("&str", size_of::<&str>()), // pointeur + longueur : « gros pointeur »
        ("&[u64]", size_of::<&[u64]>()), // pareil pour une tranche
        ("String", size_of::<String>()), // pointeur + capacité + longueur
        ("Vec<u8>", size_of::<Vec<u8>>()), // la même chose
        ("Box<u8>", size_of::<Box<u8>>()), // un pointeur, comme en C
        ("Rc<u8>", size_of::<Rc<u8>>()), // un pointeur ; les compteurs sont sur le tas
        ("Option<u8>", size_of::<Option<u8>>()), // un octet de plus pour le Some/None
        ("Option<u32>", size_of::<Option<u32>>()), // 8 : l'étiquette est alignée sur 4
        ("Option<Box<u8>>", size_of::<Option<Box<u8>>>()), // la niche : None = pointeur nul
        ("Option<String>", size_of::<Option<String>>()), // niche aussi
        ("Option<char>", size_of::<Option<char>>()), // niche : 0x110000 n'est pas un char
        ("Box<dyn Fn()>", size_of::<Box<dyn Fn()>>()), // pointeur + table de méthodes
        ("[u8; 100]", size_of::<[u8; 100]>()), // un tableau est en place
    ]
    .to_vec()
}

/// La taille d'un type, cherchée par son nom dans [`tailles`]. Pratique pour
/// les tests et l'affichage.
pub fn taille(nom: &str) -> Option<usize> {
    tailles()
        .into_iter()
        .find(|(n, _)| *n == nom)
        .map(|(_, t)| t)
}
