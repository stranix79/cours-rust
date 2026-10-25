// Labo 13 : le binaire todo-api, serveur et client.
// Lancé par : cargo run -- serve --adresse 127.0.0.1:8080
//         et : cargo run -- liste --url http://127.0.0.1:8080
//
// main lit les arguments (clap, chapitre 12) et appelle la bibliothèque.
// Il est fourni : il marchera dès que servir() et recuperer_todos() seront
// écrits dans lib.rs. Seul le TODO 12, en bas, est pour toi.

use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};
use todo_api::{Etat, format_tableau, recuperer_todos, servir};
use tokio::net::TcpListener;

/// API REST de tâches : la servir, ou l'interroger.
#[derive(Parser)]
#[command(name = "todo-api", version)]
struct Args {
    #[command(subcommand)]
    commande: Commande,
}

#[derive(Subcommand)]
enum Commande {
    /// Sert l'API jusqu'à Ctrl-C (ou SIGTERM).
    Serve {
        /// Adresse d'écoute.
        #[arg(long, default_value = "127.0.0.1:8080")]
        adresse: String,
    },
    /// Interroge une API qui tourne et affiche ses tâches.
    Liste {
        /// Adresse de base de l'API.
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Args::parse().commande {
        Commande::Serve { adresse } => {
            let ecoute = TcpListener::bind(&adresse)
                .await
                .with_context(|| format!("impossible d'écouter sur {adresse}"))?;
            eprintln!("écoute sur http://{}", ecoute.local_addr()?);
            servir(ecoute, Etat::nouveau(), signal_arret()).await?;
            eprintln!("arrêt propre");
        }
        Commande::Liste { url } => {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?;
            let todos = recuperer_todos(&client, &url).await?;
            print!("{}", format_tableau(&todos));
        }
    }
    Ok(())
}

/// Se termine au premier Ctrl-C. `docker stop` et `systemctl stop` envoient
/// SIGTERM, pas SIGINT : il faut écouter les deux.
async fn signal_arret() {
    // TODO 12 : sous Unix, attends aussi SIGTERM avec
    // tokio::signal::unix::signal(SignalKind::terminate()) et .recv().await,
    // et garde le premier des deux avec tokio::select! (le chapitre 13 montre
    // la version complète, avec #[cfg(unix)] pour que Windows compile).
    tokio::signal::ctrl_c().await.expect("gestionnaire Ctrl-C");
    eprintln!("signal reçu, arrêt en cours (les requêtes en cours finissent)");
}
