// Labo 10, solution : fait tourner chaque expérience et affiche ce qu'elle
// montre. Lancé par : (cd solution && cargo run)   depuis labs/10-sous-le-capot
//
// Rien à apprendre de neuf ici : main appelle la bibliothèque et affiche,
// la logique et les explications sont dans les modules.

use std::rc::Rc;
use std::sync::atomic::Ordering;

use capot::{arbre, compteur, ffi, liste::Liste, tailles};

fn main() {
    println!("== Compteur partagé (Rc<RefCell<T>>)");
    let partage = compteur::nouveau_compteur();
    println!("propriétaires au départ : {}", Rc::strong_count(&partage));
    // Un bloc : les capteurs sont détruits à sa fin, et le compteur de
    // références redescend. C'est Drop, rendu visible par strong_count.
    {
        let cuisine = compteur::Capteur::new("cuisine", &partage);
        let salon = compteur::Capteur::new("salon", &partage);
        println!("avec deux capteurs      : {}", Rc::strong_count(&partage));
        cuisine.mesurer(21);
        salon.mesurer(19);
        cuisine.mesurer(22);
        println!(
            "{} et {} ont écrit dans le même compteur",
            cuisine.nom, salon.nom
        );
    }
    println!("après leur destruction  : {}", Rc::strong_count(&partage));
    println!("compteur : {:?}", partage.borrow());
    match compteur::emprunt_pendant_lecture(&partage) {
        Ok(()) => println!("emprunt mutable pendant une lecture : accepté (?!)"),
        Err(e) => println!("emprunt mutable pendant une lecture : refusé ({e})"),
    }

    println!("\n== Liste chaînée (Option<Box<Noeud<T>>>)");
    let mut l = Liste::new();
    for mot in ["un", "deux", "trois"] {
        l.push(mot);
    }
    // iter() emprunte : on peut parcourir sans consommer.
    let mots: Vec<&str> = l.iter().copied().collect();
    println!(
        "contenu : {mots:?}, len = {}, tête = {:?}",
        l.len(),
        l.peek()
    );
    println!("pop : {:?}, reste {}", l.pop(), l.len());
    let mut grande = Liste::new();
    for i in 0..1_000_000u32 {
        grande.push(i);
    }
    println!("un million de maillons : len = {}", grande.len());
    drop(grande); // le Drop itératif : pas de débordement de pile
    println!("un million de maillons libérés sans exploser la pile");

    println!("\n== Arbre (Rc vers le bas, Weak vers le haut)");
    let avant = arbre::DETRUITS.load(Ordering::Relaxed);
    {
        let racine = arbre::nouveau("racine");
        let etc = arbre::nouveau("etc");
        let nginx = arbre::nouveau("nginx");
        arbre::ajouter_enfant(&racine, &etc);
        arbre::ajouter_enfant(&etc, &nginx);
        println!("chemin : {}", arbre::chemin(&nginx));
        println!(
            "etc : {} fort(s), {} faible(s)",
            Rc::strong_count(&etc),
            Rc::weak_count(&etc)
        );
    }
    let apres = arbre::DETRUITS.load(Ordering::Relaxed);
    println!("nœuds détruits en sortant du bloc : {}", apres - avant);

    println!("\n== FFI (libc)");
    println!("strlen(\"bonjour\") = {}", ffi::longueur_c(c"bonjour"));
    println!(
        "strlen(\"été\")     = {} (des octets, pas des lettres)",
        ffi::longueur_c(c"été")
    );
    println!(
        "getpid() = {}, std::process::id() = {}",
        ffi::pid(),
        std::process::id()
    );

    println!("\n== Tailles (size_of, en octets)");
    for (nom, taille) in tailles::tailles() {
        println!("{nom:<18} {taille:>4}");
    }
}
