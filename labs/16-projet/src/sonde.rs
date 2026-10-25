// Labo 16 : sonde.rs, sonder des ports TCP en parallèle.
//
// C'est le worker pool du labo 11, presque ligne pour ligne : des threads de
// la bibliothèque standard, un canal mpsc de tâches derrière un Mutex, un
// canal de résultats, et thread::scope pour emprunter les cibles. Il reste
// synchrone exprès : le service async l'appelle dans
// tokio::task::spawn_blocking (lib.rs), et ce code n'a pas à savoir que tokio
// existe. Du code bloquant éprouvé se réutilise tel quel dans un programme
// async, à condition de le mettre sur un thread où il a le droit de bloquer.

use std::net::{TcpStream, ToSocketAddrs}; // la connexion et la résolution de nom
use std::sync::atomic::{AtomicBool, Ordering}; // le drapeau d'arrêt partagé
use std::sync::{Arc, Mutex, mpsc}; // la file de tâches partagée par les workers
use std::thread; // thread::scope
use std::time::{Duration, Instant}; // les délais et le chronomètre

use serde::Serialize; // les mesures sortent en JSON par l'API

/// Une cible à sonder : un nom pour les humains, une adresse `hote:port`, un
/// délai de connexion. Construite par config.rs, déjà validée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cible {
    pub nom: String,
    pub adresse: String,
    pub timeout: Duration,
}

/// Le résultat d'une sonde, tel qu'il est stocké dans SQLite et renvoyé par
/// l'API. `instant` en secondes Unix : un entier se stocke, se compare et se
/// sérialise sans fuseau horaire ni bibliothèque de dates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mesure {
    pub cible: String,
    pub instant: i64,
    pub ok: bool,
    /// u32 : 49 jours en millisecondes, largement assez pour une sonde, et
    /// rusqlite le convertit sans perte vers un INTEGER SQLite.
    pub duree_ms: u32,
    /// `None` si la cible a répondu ; serde l'écrit `null` en JSON.
    pub erreur: Option<String>,
}

/// L'heure actuelle en secondes Unix. Une horloge avant 1970 (machine mal
/// réglée) donne 0 plutôt qu'une panique : la sonde doit tourner quand même.
#[must_use]
pub fn maintenant() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Tente une connexion TCP vers la cible, bornée par son délai. Si `arret`
/// est déjà levé, rend tout de suite un échec « annulé » sans toucher au
/// réseau. Une cible injoignable est une Mesure avec `ok: false`, jamais une
/// erreur du programme : le service doit tourner quand tout est en panne
/// autour de lui, c'est précisément là qu'on a besoin de lui.
#[must_use]
pub fn sonder(cible: &Cible, arret: &AtomicBool) -> Mesure {
    let instant = maintenant();
    let debut = Instant::now();
    let fin = |ok: bool, erreur: Option<String>| Mesure {
        cible: cible.nom.clone(),
        instant,
        ok,
        // as_millis rend un u128 ; try_from plafonne au lieu de tronquer.
        duree_ms: u32::try_from(debut.elapsed().as_millis()).unwrap_or(u32::MAX),
        erreur,
    };

    if arret.load(Ordering::Relaxed) {
        return fin(false, Some("annulé".to_string()));
    }
    // "hote:port" implémente ToSocketAddrs : résolution DNS comprise, et une
    // IPv6 entre crochets ("[::1]:22") marche aussi.
    let adresse = match cible.adresse.to_socket_addrs() {
        Ok(mut adresses) => match adresses.next() {
            Some(a) => a,
            None => return fin(false, Some("aucune adresse".to_string())),
        },
        Err(e) => return fin(false, Some(e.to_string())),
    };
    // La connexion est fermée tout de suite (le TcpStream est détruit à la
    // fin du match) : on veut savoir si le port répond, pas lui parler.
    match TcpStream::connect_timeout(&adresse, cible.timeout) {
        Ok(_flux) => fin(true, None),
        Err(e) => fin(false, Some(e.to_string())),
    }
}

/// Sonde toutes les cibles avec au plus `parallelisme` sondes en même temps
/// et rend les mesures DANS L'ORDRE des cibles. `parallelisme == 0` vaut 1.
/// Si `arret` est levé en cours de route, les sondes restantes rendent
/// « annulé » tout de suite. Bloquant : à appeler hors de l'exécuteur async.
#[must_use]
pub fn sonder_tous(cibles: &[Cible], parallelisme: usize, arret: &Arc<AtomicBool>) -> Vec<Mesure> {
    if cibles.is_empty() {
        return Vec::new();
    }
    let n_workers = parallelisme.max(1).min(cibles.len());

    // La file des tâches transporte des indices ; le récepteur unique de mpsc
    // est partagé par les workers derrière un Mutex (labo 11).
    let (tx_taches, rx_taches) = mpsc::channel::<usize>();
    let rx_taches = Arc::new(Mutex::new(rx_taches));
    let (tx_res, rx_res) = mpsc::channel::<(usize, Mesure)>();
    for i in 0..cibles.len() {
        tx_taches.send(i).expect("récepteur des tâches vivant");
    }
    drop(tx_taches); // file fermée : recv() rendra Err quand elle sera vide

    let mut places: Vec<Option<Mesure>> = (0..cibles.len()).map(|_| None).collect();
    thread::scope(|s| {
        for _ in 0..n_workers {
            let tx_res = tx_res.clone();
            let rx_taches = Arc::clone(&rx_taches);
            s.spawn(move || {
                loop {
                    // Le verrou n'est tenu que le temps du recv(), pas
                    // pendant la sonde : sinon les workers attendraient l'un
                    // après l'autre.
                    let tache = rx_taches.lock().expect("verrou empoisonné").recv();
                    let Ok(i) = tache else { break };
                    let _ = tx_res.send((i, sonder(&cibles[i], arret)));
                }
            });
        }
        drop(tx_res); // sinon la collecte attendrait un émetteur qui ne parle plus
        for (i, m) in rx_res {
            places[i] = Some(m);
        }
    });
    places
        .into_iter()
        .map(|m| m.expect("chaque cible a sa mesure"))
        .collect()
}
