// Labo 16 : le binaire sondes. Fourni, rien à écrire ici : il marchera dès
// que les TODO de la bibliothèque seront faits.
// Lancé par : cargo run -- --config sondes.toml
//
// main ne fait que brancher : la configuration, le dépôt, l'état, la boucle
// de sondage dans une tâche, le serveur HTTP, et l'arrêt propre des deux sur
// Ctrl-C ou SIGTERM. Tout ce qui se teste est dans la bibliothèque.

use std::path::PathBuf; // le chemin du fichier de configuration
use std::sync::Arc;
use std::sync::atomic::Ordering;

use anyhow::Context; // dire QUELLE adresse n'a pas pu être ouverte
use clap::Parser; // les options dérivées de la struct
use sondes::depot::Depot;
use sondes::{Etat, api, boucle, config};
use tokio::net::TcpListener;
use tokio::sync::watch; // diffuser l'arrêt à plusieurs tâches

/// Sonde des cibles TCP en parallèle et expose leur état en JSON et pour
/// Prometheus.
#[derive(Parser)]
#[command(name = "sondes", version)]
struct Args {
    /// Le fichier de configuration.
    #[arg(long, short, env = "SONDES_CONFIG", default_value = "sondes.toml")]
    config: PathBuf,
    /// L'adresse d'écoute, qui l'emporte sur celle du fichier.
    #[arg(long, env = "SONDES_ECOUTE")]
    ecoute: Option<String>,
    /// Le fichier SQLite, qui l'emporte sur celui du fichier.
    #[arg(long, env = "SONDES_BASE")]
    base: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = config::charger(&args.config)?;
    // L'environnement et la ligne de commande l'emportent sur le fichier :
    // c'est ce qui permet au même fichier de servir en local et en conteneur.
    if let Some(ecoute) = args.ecoute {
        config.ecoute = ecoute;
    }
    if let Some(base) = args.base {
        config.base = base;
    }

    let depot = Depot::nouveau(&config.base)?;
    let ecoute = TcpListener::bind(&config.ecoute)
        .await
        .with_context(|| format!("impossible d'écouter sur {}", config.ecoute))?;
    eprintln!(
        "sondes {} : {} cibles toutes les {:?}, parallélisme {}, base {}, écoute sur http://{}",
        env!("CARGO_PKG_VERSION"),
        config.cibles.len(),
        config.intervalle,
        config.parallelisme,
        config.base,
        ecoute.local_addr()?,
    );
    let etat = Etat::nouveau(config, depot);

    // Un signal, deux tâches à arrêter : le serveur et la boucle. Un canal
    // watch garde la dernière valeur et la montre à tous ses récepteurs ;
    // chacun attend qu'elle passe à true. C'est le ctx.Done() de Go, et ce
    // que fait le CancellationToken de tokio-util.
    let (emetteur, recepteur) = watch::channel(false);
    let boucle = tokio::spawn(boucle(Arc::clone(&etat), attendre(recepteur)));

    axum::serve(ecoute, api::routeur(Arc::clone(&etat)))
        .with_graceful_shutdown(async move {
            signal_arret().await;
            eprintln!("arrêt demandé");
            // send ne peut échouer que si tous les récepteurs sont tombés :
            // la boucle serait déjà finie, rien à faire.
            let _ = emetteur.send(true);
        })
        .await?;
    // Le serveur a rendu la main : on attend la boucle, qui a reçu le même
    // signal et sort à son prochain select!.
    boucle.await?;
    eprintln!("arrêté après {} tours", etat.tours.load(Ordering::Relaxed));
    Ok(())
}

/// Se termine quand la valeur du canal passe à true (ou quand l'émetteur
/// disparaît : wait_for rend alors une erreur, et on s'arrête aussi).
async fn attendre(mut recepteur: watch::Receiver<bool>) {
    let _ = recepteur.wait_for(|arret| *arret).await;
}

/// Ctrl-C partout, et SIGTERM sous Unix (docker stop, systemctl stop,
/// Kubernetes). Le même code qu'au labo 13.
async fn signal_arret() {
    let ctrl_c = async {
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
    #[cfg(not(unix))]
    let terminer = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminer => {},
    }
}
