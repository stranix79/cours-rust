// Labo 01, solution : un programme qui dit sur quelle machine il tourne.
// Lancé par : (cd solution && cargo run)   depuis labs/01-hello
//
// Ce fichier est le point d'entrée d'un exécutable : une fonction `main` dans
// `src/main.rs`, c'est tout ce que Cargo demande. Il n'y a aucune logique
// métier, on ne fait que lire des informations que le système et la
// bibliothèque standard nous donnent, puis les afficher.

use std::env; // variables d'environnement, arguments, constantes OS et ARCH, chemin du binaire
use std::process::Command; // lancer un autre programme et lire sa sortie
use std::thread; // ici seulement pour available_parallelism()

fn main() {
    // println! est une macro (le point d'exclamation) : elle vérifie à la
    // compilation que le nombre de {} correspond au nombre d'arguments.
    // Une erreur de format est une erreur de compilation, pas un plantage.
    println!("Bonjour depuis Rust.");

    // Pas de hostname() dans la std : on demande au système. Command::new
    // construit la commande, .output() la lance et attend sa fin. Le retour
    // est un Result : Ok(sortie) si le programme a pu être lancé, Err(e)
    // sinon (commande introuvable, par exemple). .expect() prend le Ok ou
    // arrête le programme avec le message donné : acceptable dans un labo,
    // à remplacer par une vraie gestion d'erreur à partir du chapitre 8.
    let sortie = Command::new("hostname").output().expect("hostname");

    // sortie.stdout est un Vec<u8>, des octets. from_utf8_lossy les lit
    // comme de l'UTF-8 en remplaçant les octets invalides par U+FFFD, et
    // .trim() enlève le retour à la ligne final. Le résultat est emprunté à
    // `sortie`, qui reste propriétaire des octets (chapitre 4).
    let nom = String::from_utf8_lossy(&sortie.stdout);
    let nom = nom.trim();

    // {:<10} : aligné à gauche sur dix colonnes, comme le %-10s du C.
    println!("{:<10}: {}", "Machine", nom);

    // OS et ARCH sont des &'static str fixés à la compilation : si tu
    // compiles pour linux/aarch64 depuis un Mac, le binaire dira linux.
    println!(
        "{:<10}: {}/{}",
        "Système",
        env::consts::OS,
        env::consts::ARCH
    );

    // available_parallelism() renvoie Result<NonZero<usize>> : le système
    // peut ne pas savoir répondre (conteneur bridé, OS exotique). On déballe
    // le NonZero avec .get() et on retombe sur 1 en cas d'erreur.
    let coeurs = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("{:<10}: {}", "Cœurs", coeurs);

    // current_exe() rend le chemin du binaire en cours : pratique pour
    // trouver des fichiers à côté de soi. .display() formate un chemin pour
    // un humain (un Path n'est pas forcément de l'UTF-8 valide).
    match env::current_exe() {
        Ok(chemin) => println!("{:<10}: {}", "Binaire", chemin.display()),
        Err(e) => println!("{:<10}: inconnu ({e})", "Binaire"),
    }
}
