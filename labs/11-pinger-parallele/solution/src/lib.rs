// Labo 11, solution : sonder des ports TCP en parallèle, avec délai et arrêt.
// Lancé par : (cd solution && cargo test)   depuis labs/11-pinger-parallele
//
// Toute la logique est ici, testée par tests/pinger.rs ; src/main.rs ne fait
// que lire les arguments et afficher. C'est la transposition du labo 11 du
// cours Go : les goroutines deviennent des threads, les channels des
// `mpsc`, et le `context.Context` un drapeau `AtomicBool` partagé.

use std::fmt;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// Délai de connexion utilisé quand une [`Cible`] n'en a pas.
// `Duration::from_secs` est une `const fn` : on peut l'appeler dans une
// constante, évaluée à la compilation.
pub const TIMEOUT_DEFAUT: Duration = Duration::from_secs(2);

/// Un hôte et un port à sonder, avec un délai optionnel (`None` = défaut).
// Clone parce qu'on recopie la cible dans le Resultat ; PartialEq pour que
// les tests comparent avec `==` ; Debug pour les messages d'assert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cible {
    pub hote: String,
    pub port: u16,
    // Option<Duration> plutôt qu'un 0 magique comme en Go : « pas de délai
    // choisi » est un état à part, le type le dit.
    pub timeout: Option<Duration>,
}

impl fmt::Display for Cible {
    /// `"hote:port"`, avec les crochets autour d'une IPv6 (`"[::1]:22"`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Une IPv6 contient des deux-points : sans crochets, impossible de
        // savoir où commence le port. C'est ce que net.JoinHostPort fait en Go.
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
    // Une String et pas un io::Error : io::Error n'est pas Clone, et le
    // tableau n'a besoin que du texte. Le « annulé » n'est pas un io::Error
    // non plus.
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

// Avec Display et Debug, l'enum devient une vraie erreur : `?` la convertit
// en Box<dyn Error> ou en anyhow::Error (chapitre 8).
impl std::error::Error for ErreurCible {}

/// Lit `"hote:port"` ou `"[ipv6]:port"` et valide le port (1 à 65535).
pub fn parse_cible(s: &str) -> Result<Cible, ErreurCible> {
    // Deux formes : avec crochets (IPv6), ou sans. On découpe d'abord en
    // (hote, texte du port), puis on valide les deux morceaux.
    let (hote, port_texte) = if let Some(reste) = s.strip_prefix('[') {
        // "[::1]:443" : l'hôte va jusqu'au "]", le port suit le ":".
        let (hote, apres) = reste.split_once(']').ok_or(ErreurCible::SansPort)?;
        let port = apres.strip_prefix(':').ok_or(ErreurCible::SansPort)?;
        (hote, port)
    } else {
        // rsplit_once coupe au DERNIER ':' et rend None s'il n'y en a pas :
        // ok_or transforme ce None en notre erreur, et `?` la renvoie.
        let (hote, port) = s.rsplit_once(':').ok_or(ErreurCible::SansPort)?;
        if hote.contains(':') {
            // "::1:22" sans crochets : ambigu, on refuse comme Go le fait.
            return Err(ErreurCible::SansPort);
        }
        (hote, port)
    };
    if hote.is_empty() {
        return Err(ErreurCible::HoteVide);
    }
    // On lit le port en u32, plus large que u16, pour distinguer « pas un
    // nombre » (PortInvalide) de « 70000 » (PortHorsPlage). Un parse direct
    // en u16 confondrait les deux dans la même ParseIntError.
    let n: u32 = port_texte
        .parse()
        .map_err(|_| ErreurCible::PortInvalide(port_texte.to_string()))?;
    if !(1..=65535).contains(&n) {
        return Err(ErreurCible::PortHorsPlage(n));
    }
    Ok(Cible {
        hote: hote.to_string(),
        // La conversion ne peut plus échouer : on vient de vérifier la plage.
        // `as` tronquerait en silence, try_from le dit, et expect documente
        // pourquoi l'échec est impossible.
        port: u16::try_from(n).expect("port déjà vérifié"),
        timeout: None,
    })
}

/// Tente une connexion TCP vers la cible, bornée par son délai (ou
/// [`TIMEOUT_DEFAUT`]). Si `arret` est déjà levé, rend tout de suite une
/// erreur « annulé » sans toucher au réseau. La connexion est refermée
/// aussitôt : on veut savoir si le port répond, pas parler au service.
pub fn sonder(cible: &Cible, arret: &AtomicBool) -> Resultat {
    let debut = Instant::now();
    // Une petite fonction locale pour fabriquer le résultat : la cible est
    // clonée parce que le Resultat la possède, alors qu'on ne l'a qu'empruntée.
    let fin = |ok: bool, erreur: Option<String>| Resultat {
        cible: cible.clone(),
        ok,
        duree: debut.elapsed(),
        erreur,
    };

    // Le drapeau d'arrêt : un simple bool lu sans verrou. Relaxed suffit, on
    // ne protège aucune autre donnée avec ce drapeau (chapitre 11).
    if arret.load(Ordering::Relaxed) {
        return fin(false, Some("annulé".to_string()));
    }

    // ToSocketAddrs résout le nom (DNS ou /etc/hosts) et rend un itérateur
    // d'adresses. Le tuple (&str, u16) l'implémente, et il gère l'IPv6 sans
    // crochets, contrairement à une chaîne "::1:22".
    let adresse = match (cible.hote.as_str(), cible.port).to_socket_addrs() {
        Ok(mut adresses) => match adresses.next() {
            Some(a) => a,
            None => return fin(false, Some("aucune adresse".to_string())),
        },
        Err(e) => return fin(false, Some(e.to_string())),
    };

    let timeout = cible.timeout.unwrap_or(TIMEOUT_DEFAUT);
    // connect_timeout ne prend qu'UNE adresse (pas un nom) : c'est pour ça
    // qu'on a résolu avant. Le TcpStream rendu est fermé quand il sort de la
    // portée : ici tout de suite, puisqu'on ne le garde pas (Drop, chapitre 4).
    match TcpStream::connect_timeout(&adresse, timeout) {
        Ok(_flux) => fin(true, None),
        Err(e) => fin(false, Some(e.to_string())),
    }
}

/// Sonde toutes les cibles avec au plus `parallelisme` sondes en même temps
/// (worker pool) et rend les résultats DANS L'ORDRE des cibles.
/// `parallelisme == 0` vaut 1. Si `arret` est levé, les sondes restantes
/// échouent tout de suite avec « annulé ».
pub fn sonder_tous(cibles: &[Cible], parallelisme: usize, arret: Arc<AtomicBool>) -> Vec<Resultat> {
    if cibles.is_empty() {
        return Vec::new();
    }
    // max(1) : zéro worker ne finirait jamais ; min(len) : inutile de lancer
    // plus de threads que de cibles.
    let n_workers = parallelisme.max(1).min(cibles.len());

    // Le canal des tâches transporte des INDICES, pas des cibles : pas de
    // clone, et l'indice sert à remettre le résultat à sa place.
    let (tx_taches, rx_taches) = mpsc::channel::<usize>();
    // mpsc = plusieurs émetteurs, UN récepteur. Pour que N workers lisent la
    // même file, on met le récepteur derrière un Mutex : celui qui tient le
    // verrou prend la tâche suivante. Arc pour le partager entre threads.
    let rx_taches = Arc::new(Mutex::new(rx_taches));
    let (tx_res, rx_res) = mpsc::channel::<(usize, Resultat)>();

    // Le canal est non borné : on peut tout envoyer d'avance, sans thread
    // producteur (le Go en avait besoin parce que son channel était un
    // rendez-vous). send ne peut échouer que si le récepteur est tombé.
    for i in 0..cibles.len() {
        tx_taches.send(i).expect("récepteur des tâches vivant");
    }
    // On lâche l'émetteur : quand la file sera vide, recv() rendra Err et
    // les workers sortiront de leur boucle. C'est le close() du Go.
    drop(tx_taches);

    // Des emplacements vides, un par cible. Option parce qu'un résultat
    // n'existe pas encore ; on les remplit dans l'ordre d'arrivée.
    let mut places: Vec<Option<Resultat>> = (0..cibles.len()).map(|_| None).collect();

    // thread::scope : les threads lancés dedans peuvent EMPRUNTER `cibles`
    // et `arret`, parce que le scope garantit qu'ils sont tous terminés à
    // sa sortie. Sans scope, il faudrait cloner les cibles dans un Arc.
    thread::scope(|s| {
        for _ in 0..n_workers {
            // Chaque worker reçoit SON clone de l'émetteur des résultats et
            // du Arc du récepteur des tâches ; `move` les fait entrer dans
            // la fermeture. `cibles` et `arret` sont des références copiées.
            let tx_res = tx_res.clone();
            let rx_taches = Arc::clone(&rx_taches);
            let arret = &arret;
            s.spawn(move || {
                loop {
                    // Le bloc limite la durée du verrou : le MutexGuard est
                    // détruit à la fin de l'instruction, AVANT la sonde.
                    // Avec `while let Ok(i) = rx.lock().unwrap().recv()`, le
                    // verrou resterait pris pendant toute la sonde et les
                    // workers travailleraient un par un.
                    let tache = rx_taches.lock().expect("verrou empoisonné").recv();
                    let Ok(i) = tache else {
                        break; // file vide et émetteur lâché : fini
                    };
                    let r = sonder(&cibles[i], arret);
                    // Si le récepteur avait disparu, on ne pourrait rien en
                    // faire : on ignore l'erreur d'envoi.
                    let _ = tx_res.send((i, r));
                }
            });
        }
        // L'émetteur d'origine doit tomber lui aussi, sinon la boucle de
        // collecte attendrait un envoi qui ne viendra jamais (interblocage).
        drop(tx_res);

        // La collecte, dans le thread principal, pendant que les workers
        // tournent : `for` sur le récepteur s'arrête quand tous les clones
        // de tx_res sont tombés, c'est-à-dire quand tous les workers ont fini.
        for (i, r) in rx_res {
            places[i] = Some(r);
        }
    });

    // Chaque place a été remplie une fois : un None ici serait un bug du
    // worker pool, pas une erreur d'utilisation.
    places
        .into_iter()
        .map(|r| r.expect("chaque cible a son résultat"))
        .collect()
}

/// Met une durée en forme comme le `String()` d'une `time.Duration` Go
/// arrondie à la milliseconde : `"0s"`, `"1ms"`, `"504ms"`, `"2s"`,
/// `"1.5s"`, `"1m30s"`.
pub fn duree_courte(d: Duration) -> String {
    // Arrondi au plus proche : on ajoute une demi-milliseconde avant de
    // tronquer, comme Round(time.Millisecond) en Go (1500 µs donne 2 ms).
    let ms = (d.as_micros() + 500) / 1000;
    if ms == 0 {
        return "0s".to_string();
    }
    if ms < 1000 {
        return format!("{ms}ms");
    }
    // Au-delà d'une seconde : secondes avec décimales sans zéros inutiles,
    // précédées des minutes et des heures s'il y en a.
    let heures = ms / 3_600_000;
    let minutes = (ms / 60_000) % 60;
    let secondes = (ms / 1000) % 60;
    let millis = ms % 1000;
    let mut s = String::new();
    if heures > 0 {
        s.push_str(&format!("{heures}h"));
    }
    if heures > 0 || minutes > 0 {
        s.push_str(&format!("{minutes}m"));
    }
    if millis == 0 {
        s.push_str(&format!("{secondes}s"));
    } else {
        // {:03} pour 1.005s, puis on retire les zéros de droite : 1.500 → 1.5.
        let fraction = format!("{millis:03}");
        s.push_str(&format!("{secondes}.{}s", fraction.trim_end_matches('0')));
    }
    s
}

/// Met les résultats en forme, une ligne par cible après un en-tête.
/// Format de chaque ligne : `"{:<22} {:<4} {:>8}  {}\n"` avec la cible,
/// `OK` ou `KO`, la durée arrondie à la milliseconde et l'erreur (vide si OK).
pub fn format_tableau(res: &[Resultat]) -> String {
    // format! dans une String plutôt qu'un println! : la fonction est
    // testable, et main décide où ça s'écrit.
    let mut sortie = format!(
        "{:<22} {:<4} {:>8}  {}\n",
        "CIBLE", "ETAT", "DUREE", "DETAIL"
    );
    for r in res {
        let etat = if r.ok { "OK" } else { "KO" };
        // as_deref : Option<String> → Option<&str>, puis "" si None. Pas de
        // clone de la chaîne d'erreur.
        let detail = r.erreur.as_deref().unwrap_or("");
        // Piège : `{:<22}` sur r.cible directement n'alignerait RIEN, parce
        // que notre Display écrit avec write! et ignore la largeur demandée.
        // On passe donc par une String, que format! sait remplir d'espaces.
        let cible = r.cible.to_string();
        sortie.push_str(&format!(
            "{:<22} {:<4} {:>8}  {}\n",
            cible,
            etat,
            duree_courte(r.duree),
            detail
        ));
    }
    sortie
}
