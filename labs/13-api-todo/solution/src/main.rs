// Labo 13, solution : le binaire todo-api, serveur et client.
// Lancé par : (cd solution && cargo run -- serve --adresse 127.0.0.1:8080)
//         et : (cd solution && cargo run -- liste --url http://127.0.0.1:8080)
//
// main ne fait que lire les arguments (clap, chapitre 12) et appeler la
// bibliothèque. Tout ce qui mérite un test est dans lib.rs.

use std::time::Duration; // le délai du client HTTP

use anyhow::Context; // pour dire QUELLE adresse n'a pas pu être ouverte
use clap::{Parser, Subcommand}; // la ligne de commande dérivée des types
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

// #[tokio::main] transforme ce `async fn main` en un main normal qui crée le
// moteur multi-thread de tokio et y exécute notre future jusqu'au bout. Sans
// lui, une fonction async ne peut pas être le point d'entrée : personne ne
// ferait avancer la future.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Args::parse().commande {
        Commande::Serve { adresse } => {
            let ecoute = TcpListener::bind(&adresse)
                .await
                .with_context(|| format!("impossible d'écouter sur {adresse}"))?;
            // local_addr() plutôt que `adresse` : si on a demandé le port 0,
            // c'est ici qu'on apprend lequel le système a choisi.
            eprintln!("écoute sur http://{}", ecoute.local_addr()?);
            servir(ecoute, Etat::nouveau(), signal_arret()).await?;
            eprintln!("arrêt propre");
        }
        Commande::Liste { url } => {
            // Un Client par programme, avec un délai : sans timeout, un
            // serveur muet nous bloquerait pour toujours.
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?;
            let todos = recuperer_todos(&client, &url).await?;
            print!("{}", format_tableau(&todos));
        }
    }
    Ok(())
}

/// Se termine au premier Ctrl-C (SIGINT) ou SIGTERM (`docker stop`,
/// `systemctl stop`). Sous Windows, seulement Ctrl-C.
async fn signal_arret() {
    let ctrl_c = async {
        // Si on ne peut pas installer le gestionnaire, mieux vaut le savoir
        // au démarrage : expect est acceptable ici.
        tokio::signal::ctrl_c().await.expect("gestionnaire Ctrl-C");
    };

    #[cfg(unix)]
    let terminer = async {
        use tokio::signal::unix::{SignalKind, signal};
        signal(SignalKind::terminate())
            .expect("gestionnaire SIGTERM")
            .recv()
            .await;
    };
    // Ailleurs, une future qui ne se termine jamais : le select! ne retiendra
    // que Ctrl-C.
    #[cfg(not(unix))]
    let terminer = std::future::pending::<()>();

    // select! attend la PREMIÈRE des deux futures et abandonne l'autre.
    tokio::select! {
        () = ctrl_c => {},
        () = terminer => {},
    }
    eprintln!("signal reçu, arrêt en cours (les requêtes en cours finissent)");
}
