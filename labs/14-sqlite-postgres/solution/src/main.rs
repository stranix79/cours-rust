// Labo 14, solution : inventaire-db, l'inventaire en ligne de commande sur
// SQLite ou PostgreSQL.
// Lancé par : (cd solution && cargo run -- --dsn sqlite:inventaire.db ajouter vis 100)
//         ou : INVENTAIRE_DSN=postgres://... cargo run -- lister
//
// main ne connaît que le trait Depot et la fonction ouvrir() : il ne sait pas,
// et n'a pas besoin de savoir, quel moteur est derrière.

use clap::{Parser, Subcommand}; // la ligne de commande dérivée des types
use inventaire_db::ouvrir;

/// Un inventaire dans SQLite ou PostgreSQL.
#[derive(Parser)]
#[command(name = "inventaire-db", version)]
struct Args {
    /// Où est la base : sqlite:fichier.db, sqlite::memory: ou postgres://...
    // env = "INVENTAIRE_DSN" : si l'option manque, clap lit la variable.
    // hide_env_values : `--help` n'affiche pas la valeur de la variable, qui
    // peut contenir un mot de passe.
    #[arg(
        long,
        env = "INVENTAIRE_DSN",
        hide_env_values = true,
        default_value = "sqlite:inventaire.db"
    )]
    dsn: String,

    #[command(subcommand)]
    commande: Commande,
}

#[derive(Subcommand)]
enum Commande {
    /// Ajoute une quantité (crée l'article s'il manque).
    Ajouter { nom: String, quantite: i64 },
    /// Retire une quantité (refuse si le stock ne suffit pas).
    Retirer { nom: String, quantite: i64 },
    /// Affiche le stock d'un article.
    Stock { nom: String },
    /// Liste tous les articles.
    Lister,
}

// main rend un anyhow::Result : une erreur s'affiche avec sa chaîne de causes
// et le code de sortie vaut 1, sans un seul unwrap.
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // `mut` : les écritures du trait prennent &mut self.
    let mut depot = ouvrir(&args.dsn)?;
    match args.commande {
        Commande::Ajouter { nom, quantite } => {
            depot.ajouter(&nom, quantite)?;
            // stock() rend Option : après un ajouter réussi, l'article existe,
            // mais on ne le suppose pas, on affiche 0 au pire.
            println!("{nom} : {} en stock", depot.stock(&nom)?.unwrap_or(0));
        }
        Commande::Retirer { nom, quantite } => {
            let reste = depot.retirer(&nom, quantite)?;
            println!("{nom} : {reste} en stock");
        }
        Commande::Stock { nom } => match depot.stock(&nom)? {
            Some(q) => println!("{nom} : {q} en stock"),
            None => {
                // Un article inconnu n'est pas une panne : un message sur
                // stderr et le code de sortie 2, que le shell peut tester.
                eprintln!("{nom} : article inconnu");
                std::process::exit(2);
            }
        },
        Commande::Lister => {
            for a in depot.lister()? {
                println!("{:<20}{:>8}", a.nom, a.quantite);
            }
        }
    }
    Ok(())
}
