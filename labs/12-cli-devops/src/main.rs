// Labo 12 : dsk, la ligne de commande (clap en derive).
// Lancé par : cargo run -- gros . --top 5
//
//   dsk gros DIR [--top 10] [--json]             les plus gros fichiers
//   dsk logs FICHIER [--niveau INFO] [--json]    filtrer un journal
//   dsk env [--prefixe DSK_] [--json]            variables d'environnement
//   dsk run -- COMMANDE ARGS...                  lancer, mesurer, rapporter
//
// Fourni presque complet : lis-le en entier, c'est le modèle d'une CLI
// clap. Un seul TODO (le 10), sur les attributs de la sous-commande env.
// Les données vont sur stdout, les bilans et les erreurs sur stderr.

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};

use dsk::{Niveau, bilan, lancer, parser_logs, plus_gros, taille_lisible, variables};

/// Petits outils DevOps : disque, journaux, environnement, commandes.
// Le commentaire /// au-dessus de la struct devient le texte de `dsk --help`,
// et ceux des champs l'aide de chaque option : la doc EST l'aide.
#[derive(Debug, Parser)]
#[command(name = "dsk", version)]
struct Cli {
    #[command(subcommand)]
    commande: Commande,
}

/// Les options communes de format. Args : un groupe d'options qu'on
/// réutilise dans plusieurs sous-commandes avec #[command(flatten)].
#[derive(Debug, Args)]
struct Format {
    /// Sortie en JSON plutôt qu'en texte
    #[arg(long)]
    json: bool,
}

/// Une variante par sous-commande ; ses champs sont ses arguments.
#[derive(Debug, Subcommand)]
enum Commande {
    /// Les plus gros fichiers d'un dossier, récursivement
    Gros {
        /// Le dossier à parcourir
        dir: PathBuf,
        /// Combien de fichiers afficher
        #[arg(short, long, default_value_t = 10)]
        top: usize,
        #[command(flatten)]
        format: Format,
    },
    /// Les lignes d'un journal à partir d'un niveau
    Logs {
        /// Le fichier journal (lignes « 2026-10-25 01:02:03 ERROR message »)
        fichier: PathBuf,
        /// Le niveau minimal : DEBUG, INFO, WARN ou ERROR
        // Niveau implémente FromStr et Display : clap s'en sert pour lire
        // la valeur et pour afficher le défaut dans l'aide.
        #[arg(short, long, default_value_t = Niveau::Info)]
        niveau: Niveau,
        #[command(flatten)]
        format: Format,
    },
    /// Les variables d'environnement qui commencent par un préfixe
    Env {
        /// Le préfixe des noms à garder
        // TODO 10 : l'option est obligatoire pour l'instant. Rends-la
        // facultative avec trois sources, par priorité : --prefixe sur la
        // ligne de commande, sinon la variable DSK_PREFIXE, sinon "DSK_".
        // Deux attributs de plus dans #[arg(...)] : default_value = "..." et
        // env = "..." (la feature "env" de clap est activée dans
        // Cargo.toml). Puis regarde `cargo run -- env --help`.
        #[arg(short, long)]
        prefixe: String,
        #[command(flatten)]
        format: Format,
    },
    /// Lance une commande et rend un rapport JSON (code, durée, sorties)
    Run {
        /// La commande et ses arguments, après `--`
        // required : au moins un mot. trailing_var_arg : tout ce qui suit
        // le premier mot est pris tel quel, même « -la » ou « --version ».
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        commande: Vec<String>,
    },
}

/// Le vrai travail. anyhow::Result : toute erreur (fichier absent, JSON,
/// écriture) remonte avec `?` et son contexte ; main l'affiche.
fn executer(cli: Cli) -> anyhow::Result<ExitCode> {
    // stdout() est déjà bufferisé par ligne ; BufWriter regroupe les
    // écritures en gros blocs, ce qui compte quand on écrit des milliers de
    // lignes dans un tuyau. lock() évite de reprendre le verrou à chaque
    // writeln!.
    let mut sortie = BufWriter::new(io::stdout().lock());

    match cli.commande {
        Commande::Gros { dir, top, format } => {
            let fichiers = plus_gros(&dir, top)?;
            if format.json {
                // to_writer_pretty écrit directement dans le flux, sans
                // String intermédiaire.
                serde_json::to_writer_pretty(&mut sortie, &fichiers)?;
                writeln!(sortie)?;
            } else {
                for f in &fichiers {
                    writeln!(
                        sortie,
                        "{:>10}  {}",
                        taille_lisible(f.taille),
                        f.chemin.display()
                    )?;
                }
            }
            // Vider stdout AVANT d'écrire sur stderr : sinon, dans un
            // terminal, le bilan (non bufferisé) passerait devant les données.
            sortie.flush()?;
            eprintln!("{} fichier(s) sous {}", fichiers.len(), dir.display());
        }
        Commande::Logs {
            fichier,
            niveau,
            format,
        } => {
            // File::open puis BufReader : lire ligne par ligne sans charger
            // tout le fichier, même s'il fait 10 Go.
            let f =
                File::open(&fichier).with_context(|| format!("ouvrir {}", fichier.display()))?;
            let entrees = parser_logs(BufReader::new(f), niveau);
            let compte = bilan(&entrees);
            if format.json {
                // json! construit une Value à la volée : pratique pour un
                // objet qui n'a pas besoin d'un type à lui.
                let rapport = serde_json::json!({ "entrees": entrees, "par_niveau": compte });
                serde_json::to_writer_pretty(&mut sortie, &rapport)?;
                writeln!(sortie)?;
            } else {
                for e in &entrees {
                    writeln!(sortie, "{} {:<5} {}", e.horodatage, e.niveau, e.message)?;
                }
            }
            let resume: Vec<String> = compte.iter().map(|(n, c)| format!("{n} {c}")).collect();
            sortie.flush()?;
            eprintln!("{} entrée(s) : {}", entrees.len(), resume.join(", "));
        }
        Commande::Env { prefixe, format } => {
            let vars = variables(&prefixe);
            if format.json {
                // Un Vec de paires deviendrait un tableau de tableaux ; une
                // BTreeMap devient un objet {"NOM": "valeur"}, trié.
                let objet: std::collections::BTreeMap<_, _> = vars.into_iter().collect();
                serde_json::to_writer_pretty(&mut sortie, &objet)?;
                writeln!(sortie)?;
            } else {
                for (nom, valeur) in &vars {
                    writeln!(sortie, "{nom}={valeur}")?;
                }
            }
        }
        Commande::Run { commande } => {
            let rapport = lancer(&commande)?;
            serde_json::to_writer_pretty(&mut sortie, &rapport)?;
            writeln!(sortie)?;
            // On vide le tampon AVANT de rendre le code : un ExitCode rendu
            // par main laisse les Drop s'exécuter, mais autant être explicite.
            sortie.flush()?;
            // Le code de la commande devient le nôtre : `dsk run -- make`
            // échoue quand make échoue. Tué par un signal : 1. Un code
            // au-delà de 255 n'existe pas sous Unix ; try_from le garde.
            let code = rapport.code.unwrap_or(1);
            return Ok(ExitCode::from(u8::try_from(code).unwrap_or(1)));
        }
    }
    sortie.flush()?;
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    // parse() lit std::env::args. Sur une erreur d'usage (option inconnue,
    // argument manquant), clap affiche le message et l'usage sur stderr et
    // sort avec le code 2 ; sur --help ou --version, il affiche et sort
    // avec 0. On n'arrive ici qu'avec une ligne de commande valide.
    let cli = Cli::parse();
    match executer(cli) {
        Ok(code) => code,
        Err(e) => {
            // {:#} : le format « alterné » d'anyhow, toute la chaîne de
            // contextes sur une ligne : « ouvrir x.log: No such file... ».
            eprintln!("dsk : {e:#}");
            ExitCode::FAILURE
        }
    }
}

// Les tests de la CLI elle-même, sans lancer de processus :
// try_parse_from prend les arguments comme une liste, et rend une erreur au
// lieu de quitter le programme.
#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn cli_coherente() {
        // debug_assert vérifie la définition elle-même : deux options avec la
        // même lettre courte, un défaut invalide... Un test gratuit.
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn gros_valeurs_par_defaut() {
        let cli = Cli::try_parse_from(["dsk", "gros", "/tmp"]).unwrap();
        match cli.commande {
            Commande::Gros { dir, top, format } => {
                assert_eq!(dir, PathBuf::from("/tmp"));
                assert_eq!(top, 10);
                assert!(!format.json);
            }
            autre => panic!("attendu Gros, obtenu {autre:?}"),
        }
    }

    #[test]
    fn env_prefixe_par_defaut() {
        let cli = Cli::try_parse_from(["dsk", "env", "--json"]).unwrap();
        match cli.commande {
            Commande::Env { prefixe, format } => {
                // Ce test suppose que DSK_PREFIXE n'est pas défini dans ton
                // shell : c'est le cas, sauf si tu l'as exporté exprès.
                assert_eq!(prefixe, "DSK_");
                assert!(format.json);
            }
            autre => panic!("attendu Env, obtenu {autre:?}"),
        }
    }

    #[test]
    fn logs_niveau_insensible_a_la_casse() {
        let cli = Cli::try_parse_from(["dsk", "logs", "app.log", "-n", "warn"]).unwrap();
        match cli.commande {
            Commande::Logs { niveau, .. } => assert_eq!(niveau, Niveau::Warn),
            autre => panic!("attendu Logs, obtenu {autre:?}"),
        }
    }

    #[test]
    fn run_garde_les_tirets() {
        let cli = Cli::try_parse_from(["dsk", "run", "--", "ls", "-la", "/"]).unwrap();
        match cli.commande {
            Commande::Run { commande } => assert_eq!(commande, ["ls", "-la", "/"]),
            autre => panic!("attendu Run, obtenu {autre:?}"),
        }
    }

    #[test]
    fn erreurs_d_usage_code_2() {
        // Sous-commande absente, argument manquant, niveau inconnu : trois
        // erreurs d'usage, toutes avec le code 2 des outils Unix.
        for args in [
            vec!["dsk"],
            vec!["dsk", "gros"],
            vec!["dsk", "logs", "x.log", "--niveau", "PANIQUE"],
            vec!["dsk", "run"],
        ] {
            let e = Cli::try_parse_from(&args).expect_err("erreur attendue");
            assert_eq!(e.exit_code(), 2, "{args:?} : {}", e.kind());
        }
        let e = Cli::try_parse_from(["dsk", "--help"]).unwrap_err();
        assert_eq!(
            e.kind(),
            ErrorKind::DisplayHelp,
            "--help n'est pas une faute"
        );
        assert_eq!(e.exit_code(), 0);
    }
}
