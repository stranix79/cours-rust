// Labo 15 : hostinfo, le binaire.
// Lancé par : cargo run -- [--json]   ou   make run
//
// Trois responsabilités, pas plus : lire les options, appeler la
// bibliothèque, rendre un code de sortie. Tout le reste est dans lib.rs.

use std::process::ExitCode;

use clap::Parser;

/// Affiche la machine, le système, l'architecture, les cœurs, la version et
/// l'uptime.
// TODO 6 : ajoute `version` dans #[command(...)] : clap crée alors --version
// et -V, avec la version de Cargo.toml. Vérifie avec cargo run -- --version.
#[derive(Parser)]
#[command(name = "hostinfo")]
struct Args {
    /// Sortie en JSON (pour jq, Ansible, un collecteur de logs).
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let infos = hostinfo::collecter();
    if args.json {
        // TODO 7 : serde_json::to_string_pretty(&infos), dans un match :
        // Ok(texte) -> println!, Err(e) -> eprintln! et ExitCode::FAILURE.
        // Pas d'unwrap : les lints de Cargo.toml le refusent (essaie).
        eprintln!("hostinfo : --json pas encore écrit (TODO 7)");
        return ExitCode::FAILURE;
    }
    print!("{}", hostinfo::format_texte(&infos));
    ExitCode::SUCCESS
}
