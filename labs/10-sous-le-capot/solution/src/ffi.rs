// Labo 10, solution : appeler la libc sans crate, par FFI.
// Lancé par : (cd solution && cargo test ffi)
//
// Rust parle le C sans glue : on déclare la signature de la fonction C, et
// l'éditeur de liens la trouve dans la libc, déjà liée à tout programme.
// Le compilateur ne peut rien vérifier de l'autre côté : l'appel est unsafe,
// et notre travail est de l'enfermer dans une fonction sûre dont la
// signature rend la mauvaise utilisation impossible.

//! Appels à la libc (`strlen`, `getpid`) par FFI, enveloppés dans des fonctions sûres.

use std::ffi::{CStr, c_char, c_int};

// Depuis l'édition 2024, un bloc extern s'écrit `unsafe extern` : le mot dit
// que c'est NOUS qui garantissons que ces signatures correspondent au C.
// Une erreur ici (un usize au lieu d'un int) compile, et donne un
// comportement indéfini à l'exécution.
unsafe extern "C" {
    // size_t strlen(const char *s);  size_t est usize sur toutes les
    // plateformes que Rust supporte, const char * est *const c_char.
    fn strlen(s: *const c_char) -> usize;
    // pid_t getpid(void);  pid_t est un int sur Linux et macOS. On pourrait
    // écrire `safe fn getpid() -> c_int;` (édition 2024) puisqu'elle n'a
    // aucune précondition ; on la garde unsafe pour montrer l'enveloppe.
    fn getpid() -> c_int;
}

/// La longueur en octets d'une chaîne C, calculée par le `strlen` de la libc.
///
/// La signature fait tout le travail de sécurité : un `&CStr` garantit un
/// pointeur valide, une chaîne terminée par un zéro, et vivante pendant
/// l'appel. `strlen` ne peut donc pas lire hors de la mémoire.
pub fn longueur_c(s: &CStr) -> usize {
    // SAFETY : s.as_ptr() pointe vers une chaîne terminée par NUL, valide
    // tant que `s` est emprunté, c'est-à-dire pendant tout l'appel.
    unsafe { strlen(s.as_ptr()) }
}

/// L'identifiant du processus courant, demandé à la libc.
pub fn pid() -> u32 {
    // SAFETY : getpid n'a pas de précondition et ne peut pas échouer.
    let p = unsafe { getpid() };
    // Un pid est toujours positif : la conversion ne peut pas échouer.
    // try_from le vérifie au lieu de laisser `as` tronquer en silence.
    u32::try_from(p).expect("un pid est positif")
}
