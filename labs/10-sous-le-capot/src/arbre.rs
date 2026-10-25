// Labo 10 : un arbre où chaque nœud connaît son parent.
// Lancé par : cargo test arbre
//
// Le cas d'école des cycles : le parent possède ses enfants, l'enfant doit
// pouvoir remonter à son parent. Si les deux liens étaient des Rc, parent et
// enfant se garderaient en vie mutuellement : compteurs jamais à zéro, Drop
// jamais appelé, fuite. La règle : Rc vers le bas (possession), Weak vers le
// haut (simple observation). Un compteur atomique de Drop le prouve.

//! Un arbre où les enfants sont des `Rc` et le parent un `Weak` : pas de cycle, pas de fuite.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Nombre total de nœuds détruits depuis le début du programme. Un `static`
/// atomique : modifiable sans `unsafe` (contrairement à un `static mut`),
/// et sans verrou.
pub static DETRUITS: AtomicUsize = AtomicUsize::new(0);

/// Le lien vers un nœud : possession partagée (Rc) et mutabilité intérieure
/// (RefCell), parce qu'on ajoute des enfants après la création.
pub type Lien = Rc<RefCell<Noeud>>;

/// Un nœud de l'arbre.
#[derive(Debug)]
pub struct Noeud {
    /// Le nom du nœud, une composante de chemin.
    pub nom: String,
    // Weak : ne compte pas comme propriétaire. upgrade() rend Some(Rc) si le
    // parent existe encore, None sinon. Weak::new() : la racine.
    // Le allow tombe quand chemin lit ce champ (TODO 8).
    #[allow(dead_code)]
    parent: Weak<RefCell<Noeud>>,
    enfants: Vec<Lien>,
}

// Fourni : chaque destruction de nœud incrémente le compteur global.
impl Drop for Noeud {
    fn drop(&mut self) {
        DETRUITS.fetch_add(1, Ordering::Relaxed);
    }
}

/// Un nœud seul, sans parent ni enfant.
pub fn nouveau(nom: &str) -> Lien {
    Rc::new(RefCell::new(Noeud {
        nom: nom.to_string(),
        parent: Weak::new(),
        enfants: Vec::new(),
    }))
}

/// Accroche `enfant` sous `parent`.
pub fn ajouter_enfant(parent: &Lien, enfant: &Lien) {
    // TODO 7 : deux lignes. Vers le haut, un lien faible :
    // enfant.borrow_mut().parent = Rc::downgrade(parent). Vers le bas, un
    // lien fort : parent.borrow_mut().enfants.push(Rc::clone(enfant)).
    // Question : que se passerait-il avec un Rc dans les deux sens ? Le test
    // arbre_sans_fuite y répond.
    todo!(
        "TODO 7 : ajouter {} sous {}",
        enfant.borrow().nom,
        parent.borrow().nom
    )
}

/// Le nombre d'enfants directs.
pub fn nb_enfants(noeud: &Lien) -> usize {
    noeud.borrow().enfants.len()
}

/// Le chemin de la racine jusqu'au nœud, séparé par des `/` :
/// `"racine/etc/nginx"`.
pub fn chemin(noeud: &Lien) -> String {
    // TODO 8 : un Vec de noms qui commence par celui du nœud, puis on
    // remonte : let mut courant = noeud.borrow().parent.upgrade(); et tant
    // que `while let Some(p) = courant`, on ajoute p.borrow().nom.clone()
    // et on passe à p.borrow().parent.upgrade(). Pour finir, reverse() et
    // join("/").
    todo!("TODO 8 : chemin de {}", noeud.borrow().nom)
}

// Les tests unitaires : ne les modifie pas.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_parent_est_faible() {
        let racine = nouveau("racine");
        let enfant = nouveau("etc");
        ajouter_enfant(&racine, &enfant);
        // Un fort pour la variable `racine`, aucun pour l'enfant qui la voit.
        assert_eq!(Rc::strong_count(&racine), 1);
        assert_eq!(Rc::weak_count(&racine), 1);
        // L'enfant : la variable + le Vec du parent.
        assert_eq!(Rc::strong_count(&enfant), 2);
        // Champ privé, visible depuis le module de tests enfant.
        assert!(enfant.borrow().parent.upgrade().is_some());
    }
}
