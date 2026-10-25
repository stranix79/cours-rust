// Labo 15, solution : hostinfo, la partie testable.
// Lancé par : (cd solution && cargo test), et utilisé par src/main.rs.
//
// Le programme est petit exprès : le sujet du labo est tout ce qu'il y a
// autour (lints, profil release, Makefile, Dockerfile, CI). La logique est
// quand même découpée en fonctions pures (lire_proc_uptime, format_duree...)
// pour que les tests tournent pareil sur Linux, macOS et Windows, avec des
// entrées fixes, sans dépendre de la machine.

use std::env; // les constantes OS et ARCH
use std::fmt::Write as _; // writeln! dans une String
use std::process::Command; // hostname et sysctl, quand il le faut
use std::thread; // available_parallelism()

use serde::Serialize; // pour --json

/// La version du paquet, lue dans Cargo.toml à la compilation. `env!` échoue
/// à la compilation si la variable manque : on ne peut pas livrer un binaire
/// sans version. Cargo définit `CARGO_PKG_VERSION` pour chaque compilation.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tout ce que hostinfo affiche. Serialize : serde en fait du JSON pour
/// `--json`, avec les noms de champs comme clés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Infos {
    pub machine: String,
    pub systeme: String,
    pub architecture: String,
    pub coeurs: usize,
    pub version: String,
    /// `None` quand le système ne sait pas le dire (Windows, ou un conteneur
    /// sans /proc). serde l'écrit `null` en JSON.
    pub uptime_secondes: Option<u64>,
}

/// Rassemble les informations de la machine courante.
#[must_use]
pub fn collecter() -> Infos {
    Infos {
        machine: nom_machine(),
        systeme: env::consts::OS.to_string(),
        architecture: env::consts::ARCH.to_string(),
        // available_parallelism peut échouer (cgroup exotique) : 1 au pire.
        coeurs: thread::available_parallelism().map_or(1, std::num::NonZero::get),
        version: VERSION.to_string(),
        uptime_secondes: uptime(),
    }
}

/// Le nom de la machine. Sous Linux, on lit /proc : dans une image `scratch`
/// il n'y a pas de commande `hostname` à lancer. Ailleurs, la commande.
fn nom_machine() -> String {
    if let Ok(nom) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
        return nom.trim().to_string();
    }
    Command::new("hostname")
        .output()
        .ok() // Result -> Option : une erreur devient None
        .filter(|sortie| sortie.status.success())
        .map(|sortie| String::from_utf8_lossy(&sortie.stdout).trim().to_string())
        .filter(|nom| !nom.is_empty())
        .unwrap_or_else(|| "inconnue".to_string())
}

/// Depuis combien de secondes la machine tourne, si on sait le dire.
fn uptime() -> Option<u64> {
    // Linux (et les conteneurs Linux) : un fichier, pas de processus à lancer.
    if let Ok(contenu) = std::fs::read_to_string("/proc/uptime") {
        return lire_proc_uptime(&contenu);
    }
    // macOS et les BSD : sysctl donne l'heure du démarrage, pas la durée.
    // cfg! est évalué à la compilation : sous Windows, ce bloc est mort et
    // le compilateur le retire.
    if cfg!(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd"
    )) {
        let sortie = Command::new("sysctl")
            .args(["-n", "kern.boottime"])
            .output()
            .ok()?;
        let maintenant = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        return lire_kern_boottime(&String::from_utf8_lossy(&sortie.stdout), maintenant);
    }
    None
}

/// Lit le contenu de /proc/uptime (`"12345.67 54321.00\n"` : secondes depuis
/// le démarrage, puis temps d'inactivité cumulé des cœurs) et rend la partie
/// entière du premier nombre.
#[must_use]
pub fn lire_proc_uptime(contenu: &str) -> Option<u64> {
    // split_whitespace().next() : le premier mot ; split('.') : la partie
    // entière. On évite le passage par f64, qui demanderait une conversion
    // flottant vers entier (et un avertissement pedantic à juste titre).
    let premier = contenu.split_whitespace().next()?;
    let entier = premier.split('.').next()?;
    entier.parse().ok()
}

/// Lit la sortie de `sysctl -n kern.boottime`
/// (`"{ sec = 1788357618, usec = 302303 } Wed Sep  2 16:00:18 2026"`) et
/// rend `maintenant - sec`, ou `None` si la sortie est illisible ou si le
/// démarrage est dans le futur.
#[must_use]
pub fn lire_kern_boottime(sortie: &str, maintenant: u64) -> Option<u64> {
    // Ce qui suit "sec = " jusqu'à la virgule. "usec = " contient aussi
    // "sec = ", mais split_once coupe à la PREMIÈRE occurrence : la bonne.
    let apres = sortie.split_once("sec = ")?.1;
    let nombre = apres.split(',').next()?.trim();
    let demarrage: u64 = nombre.parse().ok()?;
    // checked_sub rend None au lieu de paniquer (debug) ou de boucler
    // (release) si l'horloge est en arrière.
    maintenant.checked_sub(demarrage)
}

/// Une durée lisible : `42 s`, `5 min`, `3 h 12 min`, `53 j 3 h 32 min`.
#[must_use]
pub fn format_duree(secondes: u64) -> String {
    if secondes < 60 {
        return format!("{secondes} s");
    }
    let jours = secondes / 86_400;
    let heures = secondes % 86_400 / 3_600;
    let minutes = secondes % 3_600 / 60;
    if jours > 0 {
        format!("{jours} j {heures} h {minutes} min")
    } else if heures > 0 {
        format!("{heures} h {minutes} min")
    } else {
        format!("{minutes} min")
    }
}

/// Le texte affiché sans `--json` : une ligne par champ, libellés alignés.
#[must_use]
pub fn format_texte(infos: &Infos) -> String {
    let uptime = infos
        .uptime_secondes
        .map_or_else(|| "inconnu".to_string(), format_duree);
    // Un tableau de paires puis une boucle : une seule ligne de format pour
    // six champs, impossible de désaligner l'un d'eux.
    let lignes = [
        ("machine", infos.machine.clone()),
        ("système", infos.systeme.clone()),
        ("architecture", infos.architecture.clone()),
        ("cœurs", infos.coeurs.to_string()),
        ("version", infos.version.clone()),
        ("uptime", uptime),
    ];
    let mut sortie = String::new();
    for (libelle, valeur) in lignes {
        // {:<13} compte en caractères, pas en octets : « système » et
        // « cœurs » s'alignent malgré leurs accents. writeln! écrit dans la
        // String sans en créer une par ligne (ce que clippy reproche à un
        // .map(format!).collect()). Écrire dans une String ne peut pas
        // échouer : on ignore le Result, et c'est le seul cas où c'est sûr.
        let _ = writeln!(sortie, "{libelle:<13}: {valeur}");
    }
    sortie
}
