// Labo 16 : sondes, le projet final. lib.rs assemble les modules, porte
// l'état partagé et la boucle de sondage.
// Lancé par : cargo test (les tests sont dans tests/sondes.rs), ou via
// src/main.rs : cargo run -- --config sondes.toml
//
// L'architecture est complète : les types, les modules, les signatures, le
// worker pool, le dépôt, les routes, main. Il reste six TODO sur les
// morceaux clés, dans cet ordre : config.rs (1), depot.rs (2), api.rs (3 et
// 4), lib.rs (5 et 6). Le README du labo les détaille.
//
// Les dépendances entre modules vont dans un seul sens :
//   config -> sonde (la config fabrique des Cible)
//   depot  -> sonde (le dépôt stocke des Mesure)
//   api    -> lib (Etat), sonde, depot
//   lib    -> tout le monde ; main -> lib.
// Rien ne dépend de main, et sonde ne dépend de rien : c'est le code du
// chapitre 11, synchrone, réutilisé tel quel.

// Le squelette a des paramètres et des imports pas encore utilisés, dans
// tous les modules : on fait taire ces avertissements le temps du labo, pour
// toute la crate. Retire cette ligne à la fin.
#![allow(unused_variables, unused_imports)]

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
    // TODO 5 : sonder_tous(&etat.config.cibles, etat.config.parallelisme,
    // arret), puis etat.enregistrer(mesures.clone())?, et Ok(mesures).
    todo!("TODO 5")
}

/// La boucle de sondage : un tour tout de suite, puis un par intervalle,
/// jusqu'à ce que `arret` se termine. Rend la main dès l'arrêt, même au
/// milieu d'un tour : les sondes pas encore commencées sont annulées, celles
/// en cours finissent dans leur timeout.
pub async fn boucle<F>(etat: Arc<Etat>, arret: F)
where
    F: Future<Output = ()>,
{
    // TODO 6 : la boucle de sondage.
    // - un drapeau Arc<AtomicBool> à false, pour annuler les sondes ;
    // - tokio::time::interval(etat.config.intervalle), avec
    //   set_missed_tick_behavior(MissedTickBehavior::Delay) ;
    // - tokio::pin!(arret) pour pouvoir l'attendre à chaque tour ;
    // - loop { tokio::select! { () = &mut arret => break, _ = minuterie.tick() => {} } ... }
    // - le tour dans tokio::task::spawn_blocking(move || tour(&etat_tour, &drapeau_tour)),
    //   avec des clones des deux Arc (le move les emporte dans le thread) ;
    // - un second select! entre ce JoinHandle et &mut arret : à l'arrêt,
    //   lève le drapeau (store(true, Ordering::Relaxed)) et sors de la boucle ;
    //   une erreur du tour s'affiche avec eprintln!("tour de sondage : {e:#}").
    // Le test boucle_sonde_puis_s_arrete vérifie le rythme et l'arrêt.
    todo!("TODO 6")
}
