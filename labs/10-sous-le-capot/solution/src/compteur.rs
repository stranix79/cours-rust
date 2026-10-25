// Labo 10, solution : un compteur partagé par plusieurs capteurs.
// Lancé par : (cd solution && cargo test compteur)
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
    // Privé : on ne veut pas qu'un utilisateur remplace le compteur.
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
        Capteur {
            nom: nom.to_string(),
            // Rc::clone ne clone PAS le Compteur : il incrémente le compteur
            // de références et rend un second pointeur vers la même boîte.
            // On l'écrit Rc::clone(&x) plutôt que x.clone() pour que ça se
            // voie à la lecture : c'est bon marché.
            compteur: Rc::clone(compteur),
        }
    }

    /// Ajoute une mesure au compteur partagé.
    pub fn mesurer(&self, valeur: u64) {
        // &self suffit : borrow_mut() donne un accès mutable à travers une
        // référence partagée. RefCell vérifie à l'exécution qu'aucun autre
        // emprunt n'est en cours, et panique sinon. Le RefMut rangé dans `c`
        // est relâché quand `c` sort de la portée, à la fin de la fonction.
        let mut c = self.compteur.borrow_mut();
        c.total += valeur;
        c.mesures += 1;
    }
}

/// Démontre l'erreur que RefCell vérifie : un emprunt mutable pendant qu'un
/// emprunt en lecture est vivant. Rend l'erreur au lieu de paniquer, grâce
/// à `try_borrow_mut`.
pub fn emprunt_pendant_lecture(compteur: &Partage) -> Result<(), BorrowMutError> {
    // Un emprunt en lecture, gardé vivant dans une variable.
    let _lecture = compteur.borrow();
    // borrow_mut() paniquerait ici avec « already borrowed: BorrowMutError ».
    // try_borrow_mut() rend un Result : Err, puisque _lecture vit encore.
    // map(|_| ()) jette le RefMut en cas de succès (qui n'arrive pas ici).
    compteur.try_borrow_mut().map(|_| ())
    // _lecture est relâchée ici, à la fin de la fonction.
}
