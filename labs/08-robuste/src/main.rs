// Labo 08 : lit une configuration et l'affiche, ou dit pourquoi elle est refusée.
// Lancé par : cargo run -- exemple.conf   (ou invalide.conf, absent.conf)
//
// Le binaire : ici on n'a plus de variantes à tester, on veut un message
// complet pour l'humain et un code de sortie pour le shell. D'où anyhow, et
// un main minuscule qui traduit le résultat de run.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use robuste::{ErreurConfig, charger};

/// Toute la logique, avec des ? partout.
fn run(chemin: PathBuf) -> Result<()> {
    // TODO 8 : remplace cette ligne par un appel à charger(&chemin) suivi de
    // .with_context(|| format!("configuration de la sonde ({})", chemin.display()))?
    // puis affiche les quatre champs comme dans la sortie attendue du README
    // (Duration s'affiche bien avec {:?}, verbeux en « oui » ou « non »).
    let _ = (charger, chemin);
    Err(anyhow::anyhow!("TODO 8")).context("run n'est pas encore écrit")
}

fn main() -> ExitCode {
    // Sans argument : un message d'usage et le code 2.
    let Some(chemin) = env::args_os().nth(1) else {
        eprintln!("usage : robuste FICHIER.conf");
        return ExitCode::from(2);
    };

    match run(PathBuf::from(chemin)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // TODO 9 : {e} n'affiche que la couche du dessus. Passe à {e:#}
            // pour toute la chaîne de causes. Puis rends ExitCode::from(2) si
            // e.downcast_ref::<ErreurConfig>() trouve une erreur de
            // configuration dans la chaîne, ExitCode::FAILURE sinon.
            eprintln!("erreur : {e}");
            let _ = e.downcast_ref::<ErreurConfig>();
            ExitCode::FAILURE
        }
    }
}
