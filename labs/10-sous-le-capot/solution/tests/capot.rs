// Labo 10 : les tests d'intégration de capot. Identiques dans le squelette
// et dans la solution. Chaque groupe correspond à un module.

use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use capot::{arbre, compteur, ffi, liste::Liste, tailles};

// ---------- compteur ----------

#[test]
fn compteur_partage_entre_capteurs() {
    let partage = compteur::nouveau_compteur();
    assert_eq!(Rc::strong_count(&partage), 1);
    let a = compteur::Capteur::new("a", &partage);
    let b = compteur::Capteur::new("b", &partage);
    assert_eq!(
        Rc::strong_count(&partage),
        3,
        "un Rc par capteur, plus le nôtre"
    );
    a.mesurer(10);
    b.mesurer(5);
    a.mesurer(1);
    assert_eq!(
        *partage.borrow(),
        compteur::Compteur {
            total: 16,
            mesures: 3
        }
    );
    drop(a);
    assert_eq!(
        Rc::strong_count(&partage),
        2,
        "Drop d'un capteur : un de moins"
    );
    drop(b);
    assert_eq!(Rc::strong_count(&partage), 1);
}

#[test]
fn compteur_emprunt_refuse_pendant_une_lecture() {
    let partage = compteur::nouveau_compteur();
    let erreur = compteur::emprunt_pendant_lecture(&partage)
        .expect_err("un emprunt mutable pendant une lecture doit être refusé");
    assert!(erreur.to_string().contains("already"), "message : {erreur}");
    // Après la fonction, la lecture est relâchée : on peut écrire.
    assert!(partage.try_borrow_mut().is_ok());
}

// ---------- liste ----------

#[test]
fn liste_push_pop_peek_len() {
    let mut l = Liste::new();
    assert!(l.is_empty());
    assert_eq!(l.pop(), None);
    assert_eq!(l.peek(), None);
    l.push(1);
    l.push(2);
    l.push(3);
    assert_eq!(l.len(), 3);
    assert_eq!(l.peek(), Some(&3));
    assert_eq!(l.pop(), Some(3), "LIFO : le dernier empilé sort le premier");
    assert_eq!(l.pop(), Some(2));
    assert_eq!(l.len(), 1);
    l.push(4);
    assert_eq!(l.pop(), Some(4));
    assert_eq!(l.pop(), Some(1));
    assert_eq!(l.pop(), None);
    assert!(l.is_empty());
}

#[test]
fn liste_iter_par_reference() {
    let mut l = Liste::new();
    for mot in ["a", "b", "c"] {
        l.push(mot.to_string());
    }
    let vus: Vec<&String> = l.iter().collect();
    assert_eq!(vus, ["c", "b", "a"]);
    // iter() a emprunté, pas consommé : la liste est intacte.
    assert_eq!(l.len(), 3);
    assert_eq!(l.iter().map(|s| s.len()).sum::<usize>(), 3);
    assert_eq!(Liste::<u8>::default().iter().next(), None);
}

#[test]
fn liste_libere_ses_valeurs() {
    // Des Rc dans la liste : leur compteur dit si la liste les a libérés.
    let valeur = Rc::new(42);
    let mut l = Liste::new();
    for _ in 0..1000 {
        l.push(Rc::clone(&valeur));
    }
    assert_eq!(Rc::strong_count(&valeur), 1001);
    l.pop();
    assert_eq!(
        Rc::strong_count(&valeur),
        1000,
        "pop rend la valeur, détruite ici"
    );
    drop(l);
    assert_eq!(
        Rc::strong_count(&valeur),
        1,
        "Drop de la liste : tout est libéré"
    );
}

// ---------- arbre ----------

// Les tests tournent en parallèle, dans des threads, et DETRUITS est global :
// sans ce verrou, un test compterait les nœuds détruits par un autre.
static ARBRES: Mutex<()> = Mutex::new(());

#[test]
fn arbre_chemin_remonte_jusqu_a_la_racine() {
    let _verrou = ARBRES.lock().unwrap_or_else(|e| e.into_inner());
    let racine = arbre::nouveau("racine");
    let etc = arbre::nouveau("etc");
    let nginx = arbre::nouveau("nginx");
    arbre::ajouter_enfant(&racine, &etc);
    arbre::ajouter_enfant(&etc, &nginx);
    assert_eq!(arbre::chemin(&nginx), "racine/etc/nginx");
    assert_eq!(arbre::chemin(&racine), "racine");
    assert_eq!(arbre::nb_enfants(&racine), 1);
    assert_eq!(arbre::nb_enfants(&nginx), 0);
    // chemin() ne garde aucun Rc : les compteurs sont revenus à leur valeur.
    assert_eq!(Rc::strong_count(&racine), 1);
    assert_eq!(
        Rc::strong_count(&etc),
        2,
        "la variable + le Vec de la racine"
    );
    assert_eq!(Rc::weak_count(&etc), 1, "le parent de nginx");
}

#[test]
fn arbre_sans_fuite() {
    let _verrou = ARBRES.lock().unwrap_or_else(|e| e.into_inner());
    let avant = arbre::DETRUITS.load(Ordering::SeqCst);
    {
        let racine = arbre::nouveau("racine");
        for nom in ["bin", "etc", "usr"] {
            let enfant = arbre::nouveau(nom);
            arbre::ajouter_enfant(&racine, &enfant);
            let petit = arbre::nouveau("local");
            arbre::ajouter_enfant(&enfant, &petit);
        }
        assert_eq!(arbre::nb_enfants(&racine), 3);
        // Tout est vivant : rien n'est encore détruit.
        assert_eq!(arbre::DETRUITS.load(Ordering::SeqCst), avant);
    }
    // Sept nœuds (la racine, trois enfants, trois petits-enfants) : si le
    // lien vers le parent était un Rc, aucun ne serait détruit ici.
    assert_eq!(arbre::DETRUITS.load(Ordering::SeqCst) - avant, 7);
}

#[test]
fn arbre_parent_disparu() {
    let _verrou = ARBRES.lock().unwrap_or_else(|e| e.into_inner());
    let enfant = arbre::nouveau("orphelin");
    {
        let parent = arbre::nouveau("parent");
        arbre::ajouter_enfant(&parent, &enfant);
        assert_eq!(arbre::chemin(&enfant), "parent/orphelin");
    }
    // Le parent est détruit (il ne vivait que par la variable du bloc) :
    // le Weak de l'enfant ne s'upgrade plus, le chemin s'arrête à lui.
    assert_eq!(arbre::chemin(&enfant), "orphelin");
}

// ---------- ffi ----------

#[test]
fn ffi_strlen() {
    assert_eq!(ffi::longueur_c(c"bonjour"), 7);
    assert_eq!(ffi::longueur_c(c""), 0);
    assert_eq!(ffi::longueur_c(c"été"), 5, "strlen compte des octets");
}

#[test]
fn ffi_getpid() {
    assert_eq!(ffi::pid(), std::process::id());
}

// ---------- tailles ----------

#[test]
fn tailles_la_niche() {
    // None est représenté par le pointeur nul, qu'un Box ne peut pas valoir :
    // Option<Box<T>> ne coûte rien de plus que Box<T>.
    assert_eq!(
        tailles::taille("Option<Box<u8>>"),
        tailles::taille("Box<u8>")
    );
    assert_eq!(size_of::<Option<Box<u8>>>(), size_of::<Box<u8>>(), "niche");
    assert_eq!(tailles::taille("Option<String>"), tailles::taille("String"));
    // Pas de niche dans un u8 : les 256 valeurs sont toutes valides.
    assert_eq!(tailles::taille("Option<u8>"), Some(2));
}

#[test]
fn tailles_connues() {
    let attendu = [
        ("u8", 1),
        ("u64", 8),
        ("char", 4),
        ("bool", 1),
        ("()", 0),
        ("[u8; 100]", 100),
    ];
    for (nom, t) in attendu {
        assert_eq!(tailles::taille(nom), Some(t), "{nom}");
    }
    let mot = size_of::<usize>(); // 8 sur une machine 64 bits
    assert_eq!(tailles::taille("&u8"), Some(mot));
    assert_eq!(tailles::taille("&str"), Some(2 * mot), "gros pointeur");
    assert_eq!(tailles::taille("String"), Some(3 * mot));
    assert_eq!(tailles::taille("Vec<u8>"), Some(3 * mot));
    assert_eq!(
        tailles::taille("Box<dyn Fn()>"),
        Some(2 * mot),
        "données + vtable"
    );
    assert_eq!(tailles::taille("introuvable"), None);
    assert!(tailles::tailles().len() >= 15, "au moins quinze types");
}
