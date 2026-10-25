// Labo 15 : hostinfo, la partie testable.
// Lancé par : cargo test (les tests sont dans tests/hostinfo.rs), et utilisé
// par src/main.rs.
//
// Les fonctions qui touchent la machine (nom_machine, uptime) sont fournies.
// Les fonctions pures, qui lisent un texte et rendent une valeur, sont à
// écrire : ce sont elles que les tests vérifient, sur des entrées fixes.
//
// Le programme est petit exprès : le sujet du labo est tout ce qu'il y a
// autour (lints, profil release, Makefile, Dockerfile, CI). La logique est
// quand même découpée en fonctions pures (lire_proc_uptime, format_duree...)
// pour que les tests tournent pareil sur Linux, macOS et Windows, avec des
// entrées fixes, sans dépendre de la machine.

// Le squelette a des paramètres et un import pas encore utilisés : on fait
// taire ces avertissements le temps du labo. Retire cette ligne à la fin.
#![allow(unused_variables, unused_imports)]

use std::env; // les constantes OS et ARCH
use std::fmt::Write as _; // writeln! dans une String
use std::process::Command; // hostname et sysctl, quand il le faut
use std::thread; // available_parallelism()

use serde::Serialize; // pour --json

/// La version du paquet, lue dans Cargo.toml à la compilation. `env!` échoue
/// à la compilation si la variable manque : on ne peut pas livrer un binaire
/// sans version. Cargo définit `CARGO_PKG_VERSION` pour chaque compilation.
// TODO 5 : remplace "0.0.0" par env!("CARGO_PKG_VERSION"). Cargo définit
// cette variable à chaque compilation avec la version de Cargo.toml ; env!
// la lit à la compilation et en fait un &'static str.
pub const VERSION: &str = "0.0.0";

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
    // TODO 1 : le premier mot (split_whitespace().next()?), sa partie avant
    // le point (split('.').next()?), puis .parse().ok(). Un "-3.5" doit
    // donner None : u64 ne parse pas un signe moins, c'est gratuit.
    todo!("TODO 1")
}

/// Lit la sortie de `sysctl -n kern.boottime`
/// (`"{ sec = 1788357618, usec = 302303 } Wed Sep  2 16:00:18 2026"`) et
/// rend `maintenant - sec`, ou `None` si la sortie est illisible ou si le
/// démarrage est dans le futur.
#[must_use]
pub fn lire_kern_boottime(sortie: &str, maintenant: u64) -> Option<u64> {
    // TODO 2 : split_once("sec = ")?.1 donne ce qui suit le PREMIER
    // "sec = " ; coupe à la virgule, trim(), parse en u64 avec .ok()?, puis
    // maintenant.checked_sub(demarrage) (None si le démarrage est dans le
    // futur, au lieu d'un dépassement).
    todo!("TODO 2")
}

/// Une durée lisible : `42 s`, `5 min`, `3 h 12 min`, `53 j 3 h 32 min`.
#[must_use]
pub fn format_duree(secondes: u64) -> String {
    // TODO 3 : moins de 60 s -> "42 s". Sinon jours = s / 86_400,
    // heures = s % 86_400 / 3_600, minutes = s % 3_600 / 60, et trois cas :
    // "53 j 3 h 32 min", "3 h 12 min", "5 min". Lis le test durees.
    todo!("TODO 3")
}

/// Le texte affiché sans `--json` : une ligne par champ, libellés alignés.
#[must_use]
pub fn format_texte(infos: &Infos) -> String {
    // TODO 4 : une ligne par champ, dans l'ordre machine, système,
    // architecture, cœurs, version, uptime ; le libellé aligné à gauche
    // sur 13 caractères ({libelle:<13}), puis ": " et la valeur. L'uptime
    // absent s'affiche "inconnu", présent avec format_duree. Piste : un
    // tableau de paires (libellé, valeur) et une boucle avec writeln! dans
    // une String (use std::fmt::Write est déjà là).
    todo!("TODO 4")
}
