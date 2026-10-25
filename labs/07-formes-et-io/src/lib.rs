// Labo 07 : des formes, un trait, des génériques et des objets trait.
// Rôle du fichier : toute la logique testable ; main.rs l'appelle, et
// tests/formes.rs la vérifie. Les traits, les types et les signatures sont
// écrits ; à toi les corps, dans l'ordre des TODO.
// Lancé par : cargo test   (tout est rouge au départ, mais tout compile)
//
// Trois parties :
//   1. le trait Forme et ses trois implémentations, avec Display et From ;
//   2. des fonctions sur des formes, en générique et en dyn, pour comparer ;
//   3. un trait Sortie (console ou tampon de test) et un Compteur qui
//      implémente Iterator.

use std::fmt;

/// Ce que toute forme sait faire. Trois méthodes obligatoires, une par défaut.
pub trait Forme {
    /// Le nom court : `cercle`, `rectangle`, `triangle`.
    fn nom(&self) -> String;
    fn aire(&self) -> f64;
    fn perimetre(&self) -> f64;

    /// Une ligne de description : `rectangle : aire 12.00, périmètre 14.00`.
    fn decrire(&self) -> String {
        // TODO 5 : la méthode par défaut. format! avec self.nom(), self.aire()
        // et self.perimetre(), les deux nombres avec {:.2}. Elle ne sait pas
        // quel type est derrière self, et n'en a pas besoin.
        todo!("TODO 5")
    }
}

/// Un cercle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cercle {
    pub rayon: f64,
}

/// Un rectangle, constructible depuis un couple `(largeur, hauteur)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub largeur: f64,
    pub hauteur: f64,
}

/// Un triangle défini par ses trois côtés. Champs privés : la seule façon
/// d'en obtenir un est `Triangle::new`, qui vérifie que les côtés en font un.
// Tant que Triangle::new et l'impl Forme ne sont que des todo!(), personne ne
// lit les champs, et le compilateur le signale. Retire cette ligne à la fin.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    a: f64,
    b: f64,
    c: f64,
}

impl Triangle {
    /// `None` si un côté est nul ou négatif, ou si un côté est plus long que
    /// la somme des deux autres (inégalité triangulaire). Un triangle plat
    /// (1, 2, 3) est accepté.
    pub fn new(a: f64, b: f64, c: f64) -> Option<Triangle> {
        // TODO 3 : deux booléens (côtés > 0.0, et a <= b + c pour chacun des
        // trois côtés), puis `(ok).then_some(Triangle { a, b, c })`, qui rend
        // Some si vrai et None sinon. Un NaN fait échouer toute comparaison :
        // il est rejeté sans test spécial.
        let _ = (a, b, c);
        todo!("TODO 3")
    }
}

impl Forme for Cercle {
    // TODO 1 : nom "cercle", aire PI * r * r, périmètre 2 * PI * r.
    // PI est dans std::f64::consts (ajoute le use en haut du fichier).
    // Pas de decrire ici : le cercle garde la méthode par défaut.
    fn nom(&self) -> String {
        todo!("TODO 1")
    }

    fn aire(&self) -> f64 {
        todo!("TODO 1")
    }

    fn perimetre(&self) -> f64 {
        todo!("TODO 1")
    }
}

impl Forme for Rectangle {
    // TODO 2 : nom "rectangle", aire largeur * hauteur, périmètre
    // 2 * (largeur + hauteur).
    fn nom(&self) -> String {
        todo!("TODO 2")
    }

    fn aire(&self) -> f64 {
        todo!("TODO 2")
    }

    fn perimetre(&self) -> f64 {
        todo!("TODO 2")
    }
}

impl Forme for Triangle {
    // TODO 4 : nom "triangle", périmètre a + b + c, et l'aire par la formule
    // de Héron : s = périmètre / 2, aire = (s * (s - a) * (s - b) * (s - c)).sqrt().
    // Tu peux appeler self.perimetre() depuis aire().
    fn nom(&self) -> String {
        todo!("TODO 4")
    }

    fn aire(&self) -> f64 {
        todo!("TODO 4")
    }

    fn perimetre(&self) -> f64 {
        todo!("TODO 4")
    }
}

// TODO 6 : les trois Display, un write!(f, ...) chacun. Formats exacts (test
// affichage) : « cercle de rayon 2 », « rectangle 3 x 4.5 »,
// « triangle 3-4-5 ». Un f64 entier s'affiche sans décimale avec {}.
impl fmt::Display for Cercle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = f;
        todo!("TODO 6")
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = f;
        todo!("TODO 6")
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = f;
        todo!("TODO 6")
    }
}

impl From<(f64, f64)> for Rectangle {
    fn from(couple: (f64, f64)) -> Self {
        // TODO 7 : Rectangle { largeur: couple.0, hauteur: couple.1 }. Plus
        // élégant : déstructurer dans le paramètre, `fn from((largeur, hauteur): (f64, f64))`.
        // Une fois From écrit, `(3.0, 4.0).into()` marche tout seul.
        let _ = couple;
        todo!("TODO 7")
    }
}

/// La forme d'aire maximale, empruntée dans la tranche ; `None` si elle est vide.
pub fn plus_grande(formes: &[Box<dyn Forme>]) -> Option<&dyn Forme> {
    // TODO 8 : formes.iter().max_by(|x, y| x.aire().total_cmp(&y.aire())),
    // puis .map(|forme| forme.as_ref()) pour passer de &Box<dyn Forme> à
    // &dyn Forme. (max_by_key ne marche pas : un f64 n'est pas Ord.)
    let _ = formes;
    todo!("TODO 8")
}

/// L'aire totale de formes d'un même type (générique : une version compilée
/// par type).
pub fn aire_totale<T: Forme>(formes: &[T]) -> f64 {
    // TODO 9 : iter, map vers aire(), sum.
    let _ = formes;
    todo!("TODO 9")
}

/// L'aire totale de formes de types mélangés (dyn : une seule version, appel
/// par la table virtuelle).
pub fn aire_totale_dyn(formes: &[Box<dyn Forme>]) -> f64 {
    // TODO 9 (suite) : le même corps, mot pour mot. Seul le type change.
    let _ = formes;
    todo!("TODO 9")
}

/// Trie sur place, de la plus petite aire à la plus grande.
pub fn trier_par_aire(formes: &mut [Box<dyn Forme>]) {
    // TODO 10 : sort_by avec la même comparaison que plus_grande. Une
    // tranche &mut suffit pour trier : l'appelant passe `&mut son_vec`.
    let _ = formes;
    todo!("TODO 10")
}

/// Là où un rapport s'écrit.
pub trait Sortie {
    /// Écrit une ligne (sans le retour à la ligne).
    fn ecrire(&mut self, texte: &str);
}

/// La sortie standard.
pub struct Console;

/// Une sortie en mémoire, pour les tests : chaque ligne écrite est gardée.
#[derive(Debug, Default)]
pub struct Tampon(pub Vec<String>);

// TODO 11 : Console écrit avec println!, Tampon pousse une copie de la ligne
// (texte.to_string()) dans self.0.
impl Sortie for Console {
    fn ecrire(&mut self, texte: &str) {
        let _ = texte;
        todo!("TODO 11")
    }
}

impl Sortie for Tampon {
    fn ecrire(&mut self, texte: &str) {
        let _ = texte;
        todo!("TODO 11")
    }
}

/// Écrit le rapport des formes dans `sortie` :
///
/// ```text
/// 1. cercle : aire 3.14, périmètre 6.28
/// 2. rectangle : aire 12.00, périmètre 14.00
/// total : 2 formes, aire 15.14
/// plus grande : rectangle
/// ```
///
/// Une seule ligne `aucune forme` si la tranche est vide.
pub fn rapport(formes: &[Box<dyn Forme>], sortie: &mut impl Sortie) {
    // TODO 12 : `let Some(grande) = plus_grande(formes) else { ...; return; };`
    // traite la liste vide. Puis enumerate (qui compte depuis 0) et
    // decrire() pour les lignes numérotées, aire_totale_dyn pour le total,
    // grande.nom() pour la dernière ligne. Chaque ligne passe par
    // sortie.ecrire(&format!(...)).
    let _ = (formes, sortie);
    todo!("TODO 12")
}

/// Compte de `courant` (compris) à `fin` (non comprise), comme `courant..fin`.
#[derive(Debug, Clone)]
pub struct Compteur {
    pub courant: u32,
    pub fin: u32,
}

impl Compteur {
    pub fn new(debut: u32, fin: u32) -> Self {
        Compteur {
            courant: debut,
            fin,
        }
    }
}

impl Iterator for Compteur {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        // TODO 13 : None si courant >= fin ; sinon garde la valeur, avance
        // courant de 1 et rends Some(valeur). C'est la seule méthode à
        // écrire : map, filter, sum et collect viennent du trait.
        todo!("TODO 13")
    }
}
