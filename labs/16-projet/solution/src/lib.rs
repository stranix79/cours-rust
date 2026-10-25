// Labo 16, solution : sondes, le projet final. lib.rs assemble les modules,
// porte l'état partagé et la boucle de sondage.
// Lancé par : (cd solution && cargo test), ou via src/main.rs :
//   (cd solution && cargo run -- --config ../sondes.toml)
//
// Les dépendances entre modules vont dans un seul sens :
//   config -> sonde (la config fabrique des Cible)
//   depot  -> sonde (le dépôt stocke des Mesure)
//   api    -> lib (Etat), sonde, depot
//   lib    -> tout le monde ; main -> lib.
// Rien ne dépend de main, et sonde ne dépend de rien : c'est le code du
// chapitre 11, synchrone, réutilisé tel quel.

pub mod api; // les routes HTTP (axum)
pub mod config; // sondes.toml
pub mod depot; // l'historique dans SQLite (rusqlite)
pub mod sonde; // le worker pool TCP (threads de la std)

use std::collections::HashMap; // la dernière mesure, par nom de cible
use std::future::Future; // le signal d'arrêt passé à boucle()
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering}; // drapeau et compteur
use std::sync::{Arc, Mutex, RwLock}; // l'état partagé

use crate::config::Config;
use crate::depot::Depot;
use crate::sonde::{Mesure, sonder_tous};

/// L'état partagé entre la boucle de sondage et les handlers HTTP.
///
/// Trois verrous différents pour trois usages différents :
/// - `config` n'est jamais modifiée après le démarrage : pas de verrou du
///   tout, l'Arc suffit à la partager en lecture ;
/// - `derniers` est lu à chaque requête et écrit une fois par tour : un
///   RwLock, qui laisse lire en parallèle ;
/// - `depot` contient une Connection SQLite, qui n'est pas Sync : un Mutex,
///   qui ne laisse passer qu'un utilisateur à la fois et rend le tout Sync ;
/// - `tours` est un simple compteur : un atomique, sans verrou.
pub struct Etat {
    pub config: Config,
    pub depot: Mutex<Depot>,
    pub derniers: RwLock<HashMap<String, Mesure>>,
    pub tours: AtomicU64,
}

impl Etat {
    /// Un état neuf, déjà dans son Arc.
    #[must_use]
    pub fn nouveau(config: Config, depot: Depot) -> Arc<Etat> {
        Arc::new(Etat {
            config,
            depot: Mutex::new(depot),
            derniers: RwLock::new(HashMap::new()),
            tours: AtomicU64::new(0),
        })
    }

    /// Range les mesures d'un tour : dans SQLite, dans la table des
    /// dernières mesures, et compte le tour. Appelée par [`tour`], et par les
    /// tests pour injecter des mesures sans réseau.
    ///
    /// # Errors
    /// Erreur de la base ; la mémoire et le compteur sont quand même à jour,
    /// pour que /api/status et /metrics reflètent le dernier tour.
    pub fn enregistrer(&self, mesures: Vec<Mesure>) -> anyhow::Result<()> {
        // D'abord SQLite, sous son Mutex, dans un bloc : le verrou est rendu
        // à la fin du bloc, avant qu'on prenne le suivant.
        let resultat = {
            let mut depot = self.depot.lock().expect("verrou empoisonné");
            depot.enregistrer(&mesures)
        };
        {
            let mut derniers = self.derniers.write().expect("verrou empoisonné");
            for m in mesures {
                // insert remplace l'ancienne mesure de la même cible.
                derniers.insert(m.cible.clone(), m);
            }
        }
        self.tours.fetch_add(1, Ordering::Relaxed);
        resultat
    }
}

/// Un tour de sondage : toutes les cibles avec le worker pool, puis
/// l'enregistrement. Synchrone et bloquant : sonder_tous lance des threads
/// et attend des connexions TCP. On l'appelle dans spawn_blocking.
///
/// # Errors
/// Erreur de la base à l'enregistrement.
pub fn tour(etat: &Etat, arret: &Arc<AtomicBool>) -> anyhow::Result<Vec<Mesure>> {
    let mesures = sonder_tous(&etat.config.cibles, etat.config.parallelisme, arret);
    etat.enregistrer(mesures.clone())?;
    Ok(mesures)
}

/// La boucle de sondage : un tour tout de suite, puis un par intervalle,
/// jusqu'à ce que `arret` se termine. Rend la main dès l'arrêt, même au
/// milieu d'un tour : les sondes pas encore commencées sont annulées, celles
/// en cours finissent dans leur timeout.
pub async fn boucle<F>(etat: Arc<Etat>, arret: F)
where
    F: Future<Output = ()>,
{
    // Le drapeau que sonder() consulte avant chaque connexion. Un
    // AtomicBool, parce que sonder_tous est du code synchrone qui ne sait
    // rien des futures : c'est le pont entre l'arrêt async et les threads.
    let drapeau = Arc::new(AtomicBool::new(false));
    // interval() : le premier tick est immédiat, les suivants espacés.
    // Delay : si un tour dure plus que l'intervalle, le suivant attend un
    // intervalle complet au lieu de partir en rafale pour rattraper.
    let mut minuterie = tokio::time::interval(etat.config.intervalle);
    minuterie.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    // select! a besoin de reprendre la même future d'arrêt à chaque tour de
    // boucle, sans la consommer : pin! l'épingle sur la pile (une future
    // qu'on interroge plusieurs fois ne doit plus bouger en mémoire).
    tokio::pin!(arret);

    loop {
        tokio::select! {
            () = &mut arret => break,
            _ = minuterie.tick() => {}
        }
        let etat_tour = Arc::clone(&etat);
        let drapeau_tour = Arc::clone(&drapeau);
        // spawn_blocking : le tour part sur le pool de threads « bloquants »
        // de tokio, où il a le droit d'attendre. Dans l'exécuteur async, il
        // gèlerait un thread de travail et les requêtes HTTP avec lui.
        let tache = tokio::task::spawn_blocking(move || tour(&etat_tour, &drapeau_tour));
        tokio::select! {
            resultat = tache => match resultat {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => eprintln!("tour de sondage : {e:#}"),
                Err(e) => eprintln!("tour de sondage interrompu : {e}"),
            },
            () = &mut arret => {
                // On lève le drapeau : les workers annulent ce qui reste.
                // Le thread bloquant finit seul, on ne l'attend pas ici.
                drapeau.store(true, Ordering::Relaxed);
                break;
            }
        }
    }
}
