// Labo 04 : un petit inventaire, quelques opérations, l'affichage.
// Lancé par : cargo run   (panique sur le premier todo!() tant que src/lib.rs
// n'est pas complété : fais d'abord passer les tests)

use inventaire::{Article, ajouter, chercher, format_tableau, fusionner, noms, plus_cher, retirer};

fn main() {
    // Le Vec est possédé par `stock`. `mut` parce qu'on va le modifier, à
    // travers des emprunts &mut que l'on prête à ajouter et retirer.
    let mut stock: Vec<Article> = Vec::new();
    // &mut stock : un emprunt mutable qui ne dure que le temps de l'appel.
    // Juste après, stock est de nouveau à nous, et on peut le reprêter.
    ajouter(&mut stock, "clavier", 2, 4990);
    ajouter(&mut stock, "souris", 5, 1990);
    ajouter(&mut stock, "écran", 1, 18900);
    ajouter(&mut stock, "clavier", 1, 4990); // fusionne : 3 claviers

    print!("{}", format_tableau(&stock));
    println!();

    // chercher rend une référence DANS stock : on la lit tout de suite,
    // avant toute modification de stock.
    match chercher(&stock, "souris") {
        Some(a) => println!("souris : {} en stock", a.quantite),
        None => println!("souris : aucune"),
    }

    // Un Result : Ok avec le reste, ou Err avec un message.
    match retirer(&mut stock, "souris", 2) {
        Ok(reste) => println!("retrait de 2 souris, reste {reste}"),
        Err(e) => println!("erreur : {e}"),
    }
    match retirer(&mut stock, "écran", 3) {
        Ok(reste) => println!("retrait de 3 écrans, reste {reste}"),
        Err(e) => println!("erreur : {e}"),
    }
    match retirer(&mut stock, "webcam", 1) {
        Ok(reste) => println!("retrait d'une webcam, reste {reste}"),
        Err(e) => println!("erreur : {e}"),
    }

    // Option<&Article> : if let ne fait rien si l'inventaire est vide.
    if let Some(a) = plus_cher(&stock) {
        println!("le plus cher : {} ({} centimes)", a.nom, a.prix_centimes);
    }
    // noms emprunte stock : le Vec<&str> vit tant qu'on l'affiche, puis
    // il est libéré à la fin de l'instruction. Le texte n'a pas été copié.
    println!("noms : {:?}", noms(&stock));
    println!();

    // Un second inventaire, puis la fusion. fusionner PREND stock et
    // arrivage : après cette ligne, les deux noms sont inutilisables, seul
    // `tout` existe. Essaie d'afficher stock.len() juste après : E0382.
    let arrivage = vec![
        Article {
            nom: "souris".to_string(),
            quantite: 10,
            prix_centimes: 1990,
        },
        Article {
            nom: "câble".to_string(),
            quantite: 4,
            prix_centimes: 590,
        },
    ];
    let tout = fusionner(stock, arrivage);
    print!("{}", format_tableau(&tout));
}
