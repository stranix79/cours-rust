// Labo 10 : la racine de la bibliothèque capot (fournie, à lire).
// Lancé par : cargo test
//
// Cinq petites expériences, une par module, comme le labo Go du même nom :
// chacune isole un mécanisme du chapitre 10 et le rend visible par un test.

//! Expériences sous le capot de Rust : `Rc<RefCell<T>>`, liste chaînée en
//! `Box`, arbre avec `Weak`, appels à la libc par FFI, tailles des types.

#![warn(missing_docs)]

// Ici les modules sont PUBLICS (`pub mod`) : chaque expérience garde son
// espace de noms (capot::liste::Liste, capot::ffi::pid), plus parlant qu'un
// grand re-export à plat quand les modules n'ont rien à voir entre eux.
pub mod arbre;
pub mod compteur;
pub mod ffi;
pub mod liste;
pub mod tailles;
