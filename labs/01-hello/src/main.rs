// Labo 01 : un programme qui dit sur quelle machine il tourne.
// Lancé par : cargo run   (ou cargo build --release puis ./target/release/hello)

// TODO : ajoute les `use` dont tu as besoin ici, au fur et à mesure
// (un `use` inutilisé est un avertissement, pas une erreur, mais clippy
// te le reprochera). Pistes : std::env, std::process::Command, std::thread.

fn main() {
    println!("Bonjour depuis Rust.");

    // TODO 1 : le nom de la machine. La bibliothèque standard n'a pas de
    // fonction hostname() : lance la commande `hostname` avec
    // std::process::Command::new("hostname").output(), qui renvoie un
    // Result. Pour ce labo, prends la sortie avec .expect("hostname") et
    // convertis les octets en texte avec String::from_utf8_lossy(&sortie.stdout),
    // puis .trim() pour enlever le retour à la ligne.

    // TODO 2 : le système et l'architecture : std::env::consts::OS et
    // std::env::consts::ARCH sont des constantes fixées à la compilation.

    // TODO 3 : le nombre de cœurs : std::thread::available_parallelism()
    // renvoie un Result<NonZero<usize>> ; .map(|n| n.get()).unwrap_or(1)
    // donne un usize, avec 1 si le système ne sait pas répondre.

    // TODO 4 : le chemin du binaire qui tourne : std::env::current_exe()
    // (un Result<PathBuf>, affiche-le avec {:?} ou .display()).

    // Affiche chaque ligne avec println!, le libellé aligné sur 10 colonnes
    // comme dans la sortie attendue du README : println!("{:<10}: {}", "Machine", nom).
}
