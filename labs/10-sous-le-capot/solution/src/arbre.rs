// Labo 10, solution : un arbre où chaque nœud connaît son parent.
// Lancé par : (cd solution && cargo test arbre)
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
    // parent existe encore, None sinon. Weak::new() ne pointe vers rien :
    // c'est la racine.
    parent: Weak<RefCell<Noeud>>,
    enfants: Vec<Lien>,
}

// Chaque destruction de nœud incrémente le compteur global. Relaxed suffit :
// on compte, on ne synchronise rien d'autre avec ce nombre.
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
    // Vers le haut : un Weak, obtenu par Rc::downgrade, qui ne retient pas
    // le parent. Le RefMut de l'enfant ne vit que pour cette instruction.
    enfant.borrow_mut().parent = Rc::downgrade(parent);
    // Vers le bas : un Rc de plus, le parent possède l'enfant.
    parent.borrow_mut().enfants.push(Rc::clone(enfant));
}

/// Le nombre d'enfants directs.
pub fn nb_enfants(noeud: &Lien) -> usize {
    noeud.borrow().enfants.len()
}

/// Le chemin de la racine jusqu'au nœud, séparé par des `/` :
/// `"racine/etc/nginx"`.
pub fn chemin(noeud: &Lien) -> String {
    let mut noms = vec![noeud.borrow().nom.clone()];
    // On remonte avec des Rc temporaires : upgrade() rend un Rc fort tant
    // que le parent existe. Chaque tour remplace `courant` par son parent ;
    // l'ancien Rc temporaire est lâché, les compteurs reviennent à leur
    // valeur. Le borrow() ne dure que le temps de lire le champ parent.
    let mut courant = noeud.borrow().parent.upgrade();
    while let Some(p) = courant {
        noms.push(p.borrow().nom.clone());
        courant = p.borrow().parent.upgrade();
    }
    // On a empilé de la feuille vers la racine : on retourne.
    noms.reverse();
    noms.join("/")
}

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
