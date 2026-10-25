// Labo 08, solution : lit une configuration et l'affiche, ou dit pourquoi elle est refusée.
// Lancé par : (cd solution && cargo run -- exemple.conf)   depuis labs/08-robuste
//
// Le binaire, pas la bibliothèque : ici on n'a plus de variantes à tester,
// on veut un message complet pour l'humain et un code de sortie pour le shell.
// D'où anyhow, et un main minuscule qui traduit le résultat de run.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use robuste::{ErreurConfig, charger};

/// Toute la logique, avec des ? partout. Rendre une erreur plutôt que
/// quitter : les variables sont détruites normalement avant que main ne
/// choisisse le code de sortie.
fn run(chemin: PathBuf) -> Result<()> {
    // Le ? convertit l'ErreurConfig en anyhow::Error sans rien perdre : la
    // variante reste lisible par downcast_ref, sa source continue la chaîne.
    // with_context ajoute la couche du dessus : ce qu'on était en train de
    // faire. La fermeture ne s'exécute qu'en cas d'erreur.
    let config = charger(&chemin)
        .with_context(|| format!("configuration de la sonde ({})", chemin.display()))?;

    // Les résultats sur stdout, les erreurs (dans main) sur stderr.
    println!("port    : {}", config.port);
    // Duration a un Debug lisible : 3s, 1.5s, 250ms.
    println!("timeout : {:?}", config.timeout);
    println!("hotes   : {}", config.hotes.join(", "));
    println!("verbeux : {}", if config.verbeux { "oui" } else { "non" });
    Ok(())
}

fn main() -> ExitCode {
    // args_os plutôt qu'args : un chemin n'est pas forcément de l'UTF-8, et
    // args() paniquerait sur un nom de fichier exotique. let-else : sans
    // argument, on explique et on sort avec 2, le code des erreurs d'usage.
    let Some(chemin) = env::args_os().nth(1) else {
        eprintln!("usage : robuste FICHIER.conf");
        return ExitCode::from(2);
    };

    match run(PathBuf::from(chemin)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // {:#} : toute la chaîne de causes sur une ligne, séparée par des
            // deux-points. C'est ce qu'on veut lire dans un journal.
            eprintln!("erreur : {e:#}");
            // downcast_ref cherche une ErreurConfig dans la chaîne : si la
            // configuration est en cause (fichier illisible compris), code 2 ;
            // toute autre erreur, code 1.
            if e.downcast_ref::<ErreurConfig>().is_some() {
                ExitCode::from(2)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
