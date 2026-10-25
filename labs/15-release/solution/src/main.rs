// Labo 15, solution : hostinfo, le binaire.
// Lancé par : (cd solution && cargo run -- [--json])   ou   make run
//
// Trois responsabilités, pas plus : lire les options, appeler la
// bibliothèque, rendre un code de sortie. Tout le reste est dans lib.rs.

use std::process::ExitCode; // le code de sortie, sans std::process::exit

use clap::Parser; // les options dérivées de la struct

/// Affiche la machine, le système, l'architecture, les cœurs, la version et
/// l'uptime.
// version : clap ajoute --version et -V, et lit la version dans Cargo.toml
// (la même CARGO_PKG_VERSION que lib.rs). name fixe le nom affiché, quel que
// soit le nom du fichier binaire.
#[derive(Parser)]
#[command(name = "hostinfo", version)]
struct Args {
    /// Sortie en JSON (pour jq, Ansible, un collecteur de logs).
    #[arg(long)]
    json: bool,
}

// main rend un ExitCode plutôt que () : SUCCESS (0) ou FAILURE (1) pour le
// shell, et les destructeurs tournent normalement, ce que process::exit ne
// garantit pas.
fn main() -> ExitCode {
    let args = Args::parse(); // --help et --version sortent d'ici, avec le code 0
    let infos = hostinfo::collecter();
    if args.json {
        // to_string_pretty ne peut échouer que sur des types exotiques
        // (clés de map non textuelles) : on traite quand même le cas, sans
        // unwrap, puisque les lints l'interdisent.
        match serde_json::to_string_pretty(&infos) {
            Ok(texte) => println!("{texte}"),
            Err(e) => {
                eprintln!("hostinfo : {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        print!("{}", hostinfo::format_texte(&infos));
    }
    ExitCode::SUCCESS
}
