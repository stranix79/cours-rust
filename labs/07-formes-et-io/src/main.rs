// Labo 07 : construit des formes et imprime leur rapport.
// Lancé par : cargo run   (panique sur un todo! tant que lib.rs n'est pas complet)
//
// main ne calcule rien : il construit, appelle la bibliothèque, affiche.
// Le même rapport() qui écrit ici sur la console écrit dans un Tampon dans
// les tests.

use formes::{
    Cercle, Compteur, Console, Forme, Rectangle, Triangle, aire_totale, rapport, trier_par_aire,
};

fn main() {
    // Des types différents dans un même Vec : il faut un objet trait. Chaque
    // Box::new alloue la forme sur le tas, et la Box se convertit en
    // Box<dyn Forme> parce que le type du Vec est annoté.
    let mut formes: Vec<Box<dyn Forme>> = vec![
        Box::new(Rectangle::from((3.0, 4.0))),
        Box::new(Cercle { rayon: 1.5 }),
        // Triangle::new rend un Option : expect est acceptable sur des côtés
        // écrits en dur, un None serait une faute de frappe dans ce fichier.
        Box::new(Triangle::new(3.0, 4.0, 5.0).expect("triangle 3-4-5")),
        Box::new(Rectangle::from((1.0, 1.0))),
    ];

    println!("formes construites : {}", formes.len());

    // Console est une struct unité : `Console` est à la fois le type et
    // l'unique valeur. Le &mut est celui que demande rapport.
    rapport(&formes, &mut Console);

    // Trier modifie l'ordre du Vec : &mut. Les Box se déplacent dans le Vec,
    // les formes restent où elles sont sur le tas.
    trier_par_aire(&mut formes);
    let ordre: Vec<String> = formes
        .iter()
        .map(|f| format!("{} {:.1}", f.nom(), f.aire()))
        .collect();
    println!("par aire croissante : {}", ordre.join(" < "));

    // La version générique ne prend qu'un seul type à la fois : ici, un
    // tableau de Rectangle, sans Box ni table virtuelle.
    let pieces = [
        Rectangle::from((4.0, 3.0)),
        Rectangle::from((5.0, 2.5)),
        // into() : le From<(f64, f64)> de Rectangle pris dans l'autre sens.
        // Le type cible est connu, c'est celui des autres éléments du tableau.
        (2.0, 2.0).into(),
    ];
    // Ici on a les types concrets : leur Display est disponible, ce qui
    // n'est pas le cas derrière un dyn Forme (le trait ne l'exige pas).
    for piece in &pieces {
        println!("  {piece}");
    }
    println!("surface des pièces : {:.2} m²", aire_totale(&pieces));

    // Compteur implémente Iterator : map et sum viennent gratuitement.
    let somme: u32 = Compteur::new(1, 11).map(|n| n * n).sum();
    println!("somme des carrés de 1 à 10 : {somme}");
}
