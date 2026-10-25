// Labo 03, solution : le jeu du nombre à deviner, au clavier.
// Lancé par : (cd solution && cargo run)   depuis labs/03-devine-nombre
// Sans clavier : printf '50\n25\n' | cargo run
//
// Ici vivent les deux choses que la bibliothèque ne fait pas exprès : tirer
// le secret au hasard et lire l'entrée standard. Tout le reste est dans
// src/lib.rs, testé.

use devine::{Verdict, evaluer, lire_essai, message};
// Rng est un trait : c'est lui qui apporte la méthode random_range. Sans ce
// `use`, rand::rng().random_range(...) ne compile pas (E0599), même si la
// crate est dans Cargo.toml. Les traits ont leur chapitre, le 7.
use rand::Rng;
use std::io; // stdin, et le type io::Result

fn main() {
    // rand::rng() rend le générateur du thread courant, initialisé par le
    // système. 1..=100 : une plage fermée, le 100 compris.
    let secret: u32 = rand::rng().random_range(1..=100);
    println!("J'ai choisi un nombre entre 1 et 100. À toi.");

    let mut tentatives: u32 = 0;
    // Une seule String pour toutes les lignes : read_line AJOUTE à la fin,
    // il faut donc la vider à chaque tour (clear garde la mémoire allouée).
    let mut ligne = String::new();

    // loop : la boucle infinie, dont on sort par break. C'est la bonne
    // boucle quand la condition d'arrêt est au milieu du corps.
    loop {
        ligne.clear();
        // read_line(&mut ligne) : un emprunt mutable, read_line écrit dans
        // notre String sans en devenir propriétaire. Il rend un
        // io::Result<usize> : le nombre d'octets lus, 0 en fin d'entrée
        // (Ctrl-D, ou fin du tuyau). expect arrête tout sur une vraie erreur
        // d'entrée-sortie, assez rare pour un labo.
        let lus = io::stdin()
            .read_line(&mut ligne)
            .expect("lecture du clavier");
        if lus == 0 {
            println!("Abandon. C'était {secret}.");
            // break sort du loop ; main se termine juste après.
            break;
        }
        // let else : sans essai valide, on prévient et on reprend la boucle.
        let Some(essai) = lire_essai(&ligne) else {
            println!("Un nombre entre 1 et 100, s'il te plaît.");
            continue;
        };
        tentatives += 1;
        let verdict = evaluer(secret, essai);
        println!("{}", message(verdict, tentatives));
        // Verdict dérive PartialEq : on peut le comparer avec ==.
        if verdict == Verdict::Trouve {
            break;
        }
    }
}
