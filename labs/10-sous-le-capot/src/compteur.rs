// Labo 10 : un compteur partagé par plusieurs capteurs.
// Lancé par : cargo test compteur
//
// Le problème : plusieurs Capteur doivent écrire dans LE MÊME Compteur.
// Une référence &mut ne peut exister qu'en un exemplaire (chapitre 4), et
// une référence & ne permet pas d'écrire. La réponse de la bibliothèque
// standard, pour un seul thread : Rc pour partager la possession, RefCell
// pour déplacer la vérification des emprunts à l'exécution.

//! Un compteur partagé par plusieurs capteurs : `Rc<RefCell<T>>`.

use std::cell::{BorrowMutError, RefCell};
use std::rc::Rc;

/// Ce que les capteurs accumulent.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Compteur {
    /// La somme des valeurs mesurées.
    pub total: u64,
    /// Le nombre de mesures.
    pub mesures: u32,
}

/// Le type partagé : plusieurs propriétaires (Rc), mutable à travers un `&`
/// (RefCell). Un alias pour ne pas l'écrire dix fois.
pub type Partage = Rc<RefCell<Compteur>>;

/// Un capteur qui écrit dans un compteur partagé.
#[derive(Debug)]
pub struct Capteur {
    /// Le nom du capteur, pour l'affichage.
    pub nom: String,
    // Le allow tombe quand mesurer lit ce champ (TODO 2).
    #[allow(dead_code)]
    compteur: Partage,
}

/// Un compteur neuf, à zéro, prêt à être partagé.
pub fn nouveau_compteur() -> Partage {
    Rc::new(RefCell::new(Compteur::default()))
}

impl Capteur {
    /// Un capteur branché sur `compteur`. Le compteur n'est pas copié :
    /// le capteur en devient un propriétaire de plus.
    pub fn new(nom: &str, compteur: &Partage) -> Capteur {
        // TODO 1 : construis le Capteur avec nom.to_string() et
        // Rc::clone(compteur). Rc::clone ne copie pas le Compteur : il
        // incrémente le compteur de références (Rc::strong_count le montre,
        // et le test compteur_partage_entre_capteurs le vérifie).
        todo!(
            "TODO 1 : Capteur::new({nom:?}, {} propriétaires)",
            Rc::strong_count(compteur)
        )
    }

    /// Ajoute une mesure au compteur partagé.
    pub fn mesurer(&self, valeur: u64) {
        // TODO 2 : self.compteur.borrow_mut() donne un RefMut<Compteur>,
        // un accès mutable à travers &self. Ajoute valeur à total et 1 à
        // mesures.
        todo!("TODO 2 : {}.mesurer({valeur})", self.nom)
    }
}

/// Démontre l'erreur que RefCell vérifie : un emprunt mutable pendant qu'un
/// emprunt en lecture est vivant. Rend l'erreur au lieu de paniquer, grâce
/// à `try_borrow_mut`.
pub fn emprunt_pendant_lecture(compteur: &Partage) -> Result<(), BorrowMutError> {
    // TODO 3 : garde un emprunt en lecture vivant dans une variable
    // (let _lecture = compteur.borrow();), puis tente
    // compteur.try_borrow_mut() et rends son résultat, transformé avec
    // .map(|_| ()). Essaie aussi borrow_mut() à la place de try_borrow_mut()
    // et lance cargo test compteur : lis le message de la panique.
    todo!("TODO 3 : emprunt_pendant_lecture({:?})", compteur.borrow())
}
