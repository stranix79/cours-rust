// Labo 11 : sonder des ports TCP en parallèle, avec délai et arrêt.
// Lancé par : cargo test   (puis cargo run -- 127.0.0.1:22 127.0.0.1:1)
//
// Ce fichier contient toute la logique testable ; src/main.rs (fourni) ne
// fait que lire les options et afficher. Les types sont donnés, les
// fonctions sont à écrire : remplace chaque `todo!` en suivant son TODO.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

// Ajoute les `use` au fur et à mesure (un `use` inutilisé fait râler
// clippy) : std::net::{TcpStream, ToSocketAddrs}, std::sync::atomic::Ordering,
// std::sync::{Mutex, mpsc}, std::thread, std::time::Instant.

/// Délai de connexion utilisé quand une [`Cible`] n'en a pas.
pub const TIMEOUT_DEFAUT: Duration = Duration::from_secs(2);

/// Un hôte et un port à sonder, avec un délai optionnel (`None` = défaut).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cible {
    pub hote: String,
    pub port: u16,
    pub timeout: Option<Duration>,
}

impl fmt::Display for Cible {
    /// `"hote:port"`, avec les crochets autour d'une IPv6 (`"[::1]:22"`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.hote.contains(':') {
            write!(f, "[{}]:{}", self.hote, self.port)
        } else {
            write!(f, "{}:{}", self.hote, self.port)
        }
    }
}

/// L'issue d'une sonde : la cible, succès ou non, le temps mis, et le texte
/// de l'erreur (`None` si OK).
#[derive(Debug, Clone)]
pub struct Resultat {
    pub cible: Cible,
    pub ok: bool,
    pub duree: Duration,
    pub erreur: Option<String>,
}

/// Ce qui peut être faux dans une cible tapée sur la ligne de commande.
#[derive(Debug, PartialEq, Eq)]
pub enum ErreurCible {
    /// Pas de `:port` du tout.
    SansPort,
    /// Le port n'est pas un nombre (le texte fautif est gardé).
    PortInvalide(String),
    /// Le port est un nombre, mais pas entre 1 et 65535.
    PortHorsPlage(u32),
    /// Rien avant le `:`.
    HoteVide,
}

impl fmt::Display for ErreurCible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurCible::SansPort => write!(f, "port manquant (attendu hote:port)"),
            ErreurCible::PortInvalide(p) => write!(f, "port {p:?} : pas un nombre"),
            ErreurCible::PortHorsPlage(n) => write!(f, "port {n} hors plage (1 à 65535)"),
            ErreurCible::HoteVide => write!(f, "hôte vide"),
        }
    }
}

impl std::error::Error for ErreurCible {}

/// Lit `"hote:port"` ou `"[ipv6]:port"` et valide le port (1 à 65535).
pub fn parse_cible(s: &str) -> Result<Cible, ErreurCible> {
    // TODO 1 : deux formes. Si `s.strip_prefix('[')` rend Some(reste), c'est
    // une IPv6 : `reste.split_once(']')` donne l'hôte et "…:port" (le port
    // suit un ':' à retirer avec strip_prefix). Sinon, `s.rsplit_once(':')`
    // coupe au dernier ':' ; un hôte qui contient encore ':' est une IPv6
    // sans crochets, refuse-la avec SansPort. Pas de ':' du tout : SansPort
    // (`.ok_or(ErreurCible::SansPort)?` transforme un None en erreur).
    // Hôte vide : HoteVide. Lis le port en u32 avec .parse() (échec :
    // PortInvalide avec le texte), puis vérifie 1..=65535 (sinon
    // PortHorsPlage(n)) et convertis en u16 avec u16::try_from.
    todo!("TODO 1 : parse_cible({s:?})")
}

/// Tente une connexion TCP vers la cible, bornée par son délai (ou
/// [`TIMEOUT_DEFAUT`]). Si `arret` est déjà levé, rend tout de suite une
/// erreur « annulé » sans toucher au réseau. La connexion est refermée
/// aussitôt : on veut savoir si le port répond, pas parler au service.
pub fn sonder(cible: &Cible, arret: &AtomicBool) -> Resultat {
    // TODO 2 : note l'heure avec Instant::now(). Si arret.load(Ordering::Relaxed)
    // est vrai, rends un Resultat KO avec erreur Some("annulé".to_string()).
    // Sinon résous l'adresse avec (cible.hote.as_str(), cible.port).to_socket_addrs()
    // (trait ToSocketAddrs) et prends la première avec .next() ; puis
    // TcpStream::connect_timeout(&adresse, cible.timeout.unwrap_or(TIMEOUT_DEFAUT)).
    // Ok : le flux est fermé dès qu'il sort de la portée, rends ok = true.
    // Err(e) : ok = false, erreur = Some(e.to_string()). La durée est
    // debut.elapsed(), la cible est cible.clone().
    todo!("TODO 2 : sonder({cible}, arret = {arret:?})")
}

/// Sonde toutes les cibles avec au plus `parallelisme` sondes en même temps
/// (worker pool) et rend les résultats DANS L'ORDRE des cibles.
/// `parallelisme == 0` vaut 1. Si `arret` est levé, les sondes restantes
/// échouent tout de suite avec « annulé ».
pub fn sonder_tous(cibles: &[Cible], parallelisme: usize, arret: Arc<AtomicBool>) -> Vec<Resultat> {
    // TODO 3 : le worker pool du chapitre 11. Aucune cible : Vec::new().
    // Nombre de workers : parallelisme.max(1).min(cibles.len()).
    // - un canal de tâches mpsc::channel::<usize>() : envoie tous les
    //   indices 0..cibles.len(), puis drop(tx) pour que recv() finisse par
    //   rendre Err quand la file est vide ;
    // - le récepteur des tâches dans un Arc<Mutex<_>> : mpsc n'a qu'un
    //   récepteur, le Mutex le partage entre les workers ;
    // - un canal de résultats mpsc::channel::<(usize, Resultat)>() ;
    // - thread::scope(|s| { ... }) pour que les threads empruntent `cibles`
    //   et `arret` ; dans chaque s.spawn(move || ...), une boucle qui prend
    //   le verrou JUSTE le temps du recv() (let tache = rx.lock().unwrap().recv();)
    //   puis sonde hors du verrou et envoie (i, resultat) ;
    // - drop du tx des résultats d'origine, puis `for (i, r) in rx_res`
    //   range chaque résultat à sa place dans un Vec<Option<Resultat>>.
    todo!(
        "TODO 3 : sonder_tous({} cibles, parallelisme = {parallelisme}, arret = {arret:?})",
        cibles.len()
    )
}

/// Met une durée en forme comme le `String()` d'une `time.Duration` Go
/// arrondie à la milliseconde : `"0s"`, `"1ms"`, `"504ms"`, `"2s"`,
/// `"1.5s"`, `"1m30s"`.
pub fn duree_courte(d: Duration) -> String {
    // TODO 4 : arrondis à la milliseconde la plus proche :
    // (d.as_micros() + 500) / 1000. Zéro : "0s". Moins de 1000 : "{ms}ms".
    // Au-delà : heures "h" et minutes "m" s'il y en a, puis les secondes avec
    // la partie décimale sans zéros inutiles (format!("{millis:03}") puis
    // .trim_end_matches('0')) : 1500 ms donne "1.5s", 90 s donne "1m30s".
    todo!("TODO 4 : duree_courte({d:?})")
}

/// Met les résultats en forme, une ligne par cible après un en-tête.
/// Format de chaque ligne : `"{:<22} {:<4} {:>8}  {}\n"` avec la cible,
/// `OK` ou `KO`, la durée arrondie à la milliseconde et l'erreur (vide si OK).
pub fn format_tableau(res: &[Resultat]) -> String {
    // TODO 5 : commence par l'en-tête
    // format!("{:<22} {:<4} {:>8}  {}\n", "CIBLE", "ETAT", "DUREE", "DETAIL"),
    // puis une ligne par résultat avec push_str. Piège : `{:<22}` appliqué
    // directement à r.cible n'aligne rien, parce que le Display de Cible
    // écrit avec write! et ignore la largeur ; passe par r.cible.to_string().
    // Le détail : r.erreur.as_deref().unwrap_or("").
    todo!("TODO 5 : format_tableau({} résultats)", res.len())
}
