// Labo 03 : le jeu du nombre à deviner, au clavier.
// Lancé par : cargo run   Sans clavier : printf '50\n25\n' | cargo run
//
// La bibliothèque (src/lib.rs) sait juger un essai. Ce fichier tire le
// secret et fait la boucle de jeu. Complète les TODO 5 et 6 une fois les
// tests de la bibliothèque au vert.

// Les imports déjà prêts ; certains ne serviront qu'une fois les TODO faits.
#[allow(unused_imports)]
use devine::{Verdict, evaluer, lire_essai, message};
// Rng est un trait : c'est lui qui apporte la méthode random_range. Sans ce
// `use`, l'appel ne compile pas, même avec rand dans Cargo.toml.
#[allow(unused_imports)]
use rand::Rng;
#[allow(unused_imports)]
use std::io;

fn main() {
    // TODO 5 : tire le secret avec rand::rng().random_range(1..=100), dans
    // un u32 (let secret: u32 = ...). En attendant, il vaut toujours 42.
    let secret: u32 = 42;
    println!("J'ai choisi un nombre entre 1 et 100. À toi.");

    // TODO 6 : la boucle de jeu, avec `loop`. Une seule String pour toutes
    // les lignes, créée avant la boucle avec String::new() :
    // - ligne.clear(), puis io::stdin().read_line(&mut ligne) qui rend un
    //   Result<usize> (le nombre d'octets lus) : .expect("lecture") suffit ;
    // - 0 octet lu = fin de l'entrée (Ctrl-D) : affiche le secret, break ;
    // - lire_essai(&ligne) ; si None, un message et continue ;
    // - compte la tentative, evaluer, affiche message(verdict, tentatives),
    //   et break si le verdict est Verdict::Trouve.
    println!("(la boucle de jeu n'est pas encore écrite ; le secret était {secret})");
}
