// Labo 07, solution : des formes, un trait, des génériques et des objets trait.
// Lancé par : (cd solution && cargo test)   depuis labs/07-formes-et-io
//
// Rôle du fichier : toute la logique testable. Trois parties :
//   1. le trait Forme et ses trois implémentations (Cercle, Rectangle,
//      Triangle), avec Display et From ;
//   2. des fonctions sur des formes, en générique et en dyn, pour comparer ;
//   3. un trait Sortie, qui sépare le calcul d'un rapport de l'endroit où on
//      l'écrit, et un Compteur qui implémente Iterator.

use std::f64::consts::PI;
use std::fmt;

/// Ce que toute forme sait faire. Trois méthodes obligatoires, une par défaut.
pub trait Forme {
    /// Le nom court : `cercle`, `rectangle`, `triangle`.
    fn nom(&self) -> String;
    fn aire(&self) -> f64;
    fn perimetre(&self) -> f64;

    /// Une ligne de description, construite sur les trois autres méthodes :
    /// `rectangle : aire 12.00, périmètre 14.00`.
    fn decrire(&self) -> String {
        // Une méthode par défaut peut appeler les méthodes obligatoires :
        // elle ne sait pas quel type est derrière self, mais elle sait que ce
        // type les a, puisqu'il implémente Forme.
        format!(
            "{} : aire {:.2}, périmètre {:.2}",
            self.nom(),
            self.aire(),
            self.perimetre()
        )
    }
}

/// Un cercle. Copy : deux f64, rien sur le tas, la copie ne coûte rien.
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

/// Un triangle défini par ses trois côtés. Les champs sont privés : tous les
/// triplets de nombres ne font pas un triangle, et la seule façon d'en
/// obtenir un est `Triangle::new`, qui vérifie.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    a: f64,
    b: f64,
    c: f64,
}

impl Triangle {
    /// `None` si un côté est nul ou négatif, ou si un côté est plus long que
    /// la somme des deux autres (inégalité triangulaire).
    pub fn new(a: f64, b: f64, c: f64) -> Option<Triangle> {
        // Les deux contrôles en une expression. `<= 0.0` attrape aussi les
        // côtés négatifs ; un NaN fait échouer toutes les comparaisons, donc
        // l'inégalité triangulaire le rejette aussi.
        let cotes_positifs = a > 0.0 && b > 0.0 && c > 0.0;
        let inegalite = a <= b + c && b <= a + c && c <= a + b;
        // bool::then_some : Some(valeur) si vrai, None sinon. Plus court qu'un
        // if/else qui rend Some et None.
        (cotes_positifs && inegalite).then_some(Triangle { a, b, c })
    }
}

impl Forme for Cercle {
    fn nom(&self) -> String {
        "cercle".to_string()
    }

    fn aire(&self) -> f64 {
        PI * self.rayon * self.rayon
    }

    fn perimetre(&self) -> f64 {
        2.0 * PI * self.rayon
    }
    // Pas de decrire : le cercle garde la méthode par défaut du trait.
}

impl Forme for Rectangle {
    fn nom(&self) -> String {
        "rectangle".to_string()
    }

    fn aire(&self) -> f64 {
        self.largeur * self.hauteur
    }

    fn perimetre(&self) -> f64 {
        2.0 * (self.largeur + self.hauteur)
    }
}

impl Forme for Triangle {
    fn nom(&self) -> String {
        "triangle".to_string()
    }

    fn aire(&self) -> f64 {
        // Formule de Héron : avec s le demi-périmètre,
        // aire = racine de s(s-a)(s-b)(s-c). Elle ne demande que les côtés,
        // pas la hauteur. On appelle self.perimetre() : une méthode du trait
        // peut en utiliser une autre, même dans une implémentation.
        let s = self.perimetre() / 2.0;
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    }

    fn perimetre(&self) -> f64 {
        self.a + self.b + self.c
    }
}

// Display, pour un humain. Un f64 entier s'affiche sans décimales avec {} :
// 2.0 donne « 2 », 2.5 donne « 2.5 ». C'est ce qu'on veut pour des côtés.
impl fmt::Display for Cercle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cercle de rayon {}", self.rayon)
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rectangle {} x {}", self.largeur, self.hauteur)
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "triangle {}-{}-{}", self.a, self.b, self.c)
    }
}

/// `Rectangle::from((3.0, 4.0))`, ou `let r: Rectangle = (3.0, 4.0).into();`.
impl From<(f64, f64)> for Rectangle {
    // Le motif de tuple directement dans le paramètre : pas besoin de
    // couple.0 et couple.1.
    fn from((largeur, hauteur): (f64, f64)) -> Self {
        Rectangle { largeur, hauteur }
    }
    // Implémenter From donne Into gratuitement, par une implémentation
    // générale de la std : impl<T, U: From<T>> Into<U> for T.
}

/// La forme d'aire maximale, empruntée dans la tranche ; `None` si elle est vide.
pub fn plus_grande(formes: &[Box<dyn Forme>]) -> Option<&dyn Forme> {
    formes
        .iter()
        // Pas de max_by_key : la clé serait un f64, qui n'est pas Ord. max_by
        // prend une comparaison, et total_cmp en donne une sur les flottants.
        .max_by(|x, y| x.aire().total_cmp(&y.aire()))
        // iter() donne des &Box<dyn Forme> ; as_ref() les transforme en
        // &dyn Forme, ce que promet la signature. La référence rendue emprunte
        // la tranche de l'appelant (durée de vie élidée, chapitre 4).
        .map(|forme| forme.as_ref())
}

/// L'aire totale de formes **d'un même type**. Générique : le compilateur
/// écrit une version par type utilisé (une pour Cercle, une pour
/// Rectangle...), et l'appel à aire() est direct.
pub fn aire_totale<T: Forme>(formes: &[T]) -> f64 {
    formes.iter().map(|f| f.aire()).sum()
}

/// La même chose pour des formes **de types mélangés**. Une seule version
/// compilée ; chaque appel à aire() passe par la table virtuelle de la forme.
pub fn aire_totale_dyn(formes: &[Box<dyn Forme>]) -> f64 {
    // Le corps est identique, mot pour mot : c'est le type du paramètre qui
    // change la façon dont Rust compile l'appel.
    formes.iter().map(|f| f.aire()).sum()
}

/// Trie sur place, de la plus petite aire à la plus grande. Une tranche
/// mutable suffit : trier ne change pas la longueur, et l'appelant peut
/// passer `&mut v` d'un Vec comme d'un tableau.
pub fn trier_par_aire(formes: &mut [Box<dyn Forme>]) {
    // sort_by est stable : deux formes de même aire gardent leur ordre.
    // On trie les Box (des pointeurs), les formes elles-mêmes ne bougent pas
    // en mémoire.
    formes.sort_by(|x, y| x.aire().total_cmp(&y.aire()));
}

/// Là où un rapport s'écrit. Le rapport ne sait pas si c'est le terminal, un
/// fichier ou un tampon de test : il appelle ecrire, c'est tout.
pub trait Sortie {
    /// Écrit une ligne (sans le retour à la ligne, que la sortie ajoute si
    /// elle en a besoin).
    fn ecrire(&mut self, texte: &str);
}

/// La sortie standard. Une struct unité : aucune donnée, juste un type sur
/// lequel accrocher l'implémentation.
pub struct Console;

impl Sortie for Console {
    // &mut self alors que Console n'a rien à modifier : la signature est
    // celle du trait, et elle doit convenir à Tampon, qui, lui, modifie.
    fn ecrire(&mut self, texte: &str) {
        println!("{texte}");
    }
}

/// Une sortie en mémoire, pour les tests : chaque ligne écrite est gardée.
/// Le champ est public pour que le test puisse lire `tampon.0`.
#[derive(Debug, Default)]
pub struct Tampon(pub Vec<String>);

impl Sortie for Tampon {
    fn ecrire(&mut self, texte: &str) {
        // On reçoit une &str empruntée : pour la garder, il faut une copie
        // possédée.
        self.0.push(texte.to_string());
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
    // impl Sortie en argument : un générique abrégé. rapport sera compilée
    // une fois pour Console et une fois pour Tampon, sans table virtuelle.
    // Le &mut est nécessaire : écrire modifie la sortie (le Tampon grandit).
    let Some(grande) = plus_grande(formes) else {
        sortie.ecrire("aucune forme");
        return;
    };
    // enumerate compte à partir de 0 ; un rapport compte à partir de 1.
    for (i, forme) in formes.iter().enumerate() {
        sortie.ecrire(&format!("{}. {}", i + 1, forme.decrire()));
    }
    sortie.ecrire(&format!(
        "total : {} formes, aire {:.2}",
        formes.len(),
        aire_totale_dyn(formes)
    ));
    sortie.ecrire(&format!("plus grande : {}", grande.nom()));
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
    // Le type associé : ce que l'itérateur produit. Une seule réponse possible
    // pour Compteur, d'où un type associé et pas un paramètre générique.
    type Item = u32;

    // La seule méthode à écrire. Tout le reste (map, filter, sum, collect,
    // la boucle for) vient des méthodes par défaut du trait Iterator.
    fn next(&mut self) -> Option<u32> {
        // Fini : None, et on continue de rendre None si on rappelle next.
        if self.courant >= self.fin {
            return None;
        }
        let valeur = self.courant;
        self.courant += 1;
        Some(valeur)
    }
}
