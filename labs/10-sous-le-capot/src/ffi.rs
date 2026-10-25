// Labo 10 : appeler la libc sans crate, par FFI.
// Lancé par : cargo test ffi
//
// Rust parle le C sans glue : on déclare la signature de la fonction C, et
// l'éditeur de liens la trouve dans la libc, déjà liée à tout programme.
// Le compilateur ne peut rien vérifier de l'autre côté : l'appel est unsafe,
// et notre travail est de l'enfermer dans une fonction sûre dont la
// signature rend la mauvaise utilisation impossible.

//! Appels à la libc (`strlen`, `getpid`) par FFI, enveloppés dans des fonctions sûres.

use std::ffi::CStr;

// TODO 9a : déclare les deux fonctions C dans un bloc
//     unsafe extern "C" {
//         fn strlen(s: *const c_char) -> usize;   // size_t strlen(const char *s);
//         fn getpid() -> c_int;                   // pid_t getpid(void);
//     }
// (édition 2024 : le bloc s'écrit `unsafe extern`), et importe c_char et
// c_int depuis std::ffi.

/// La longueur en octets d'une chaîne C, calculée par le `strlen` de la libc.
///
/// La signature fait tout le travail de sécurité : un `&CStr` garantit un
/// pointeur valide, une chaîne terminée par un zéro, et vivante pendant
/// l'appel. `strlen` ne peut donc pas lire hors de la mémoire.
pub fn longueur_c(s: &CStr) -> usize {
    // TODO 9b : unsafe { strlen(s.as_ptr()) }, précédé d'un commentaire
    // « SAFETY : » qui dit pourquoi l'appel est correct.
    todo!("TODO 9b : longueur_c({s:?})")
}

/// L'identifiant du processus courant, demandé à la libc.
pub fn pid() -> u32 {
    // TODO 9c : unsafe { getpid() }, puis u32::try_from(p).expect(...) : un
    // pid est positif, try_from le vérifie au lieu que `as` tronque.
    todo!("TODO 9c : pid()")
}
