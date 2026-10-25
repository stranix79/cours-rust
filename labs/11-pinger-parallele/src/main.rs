// Labo 11 : pinger, sonde des ports TCP en parallèle.
// Lancé par : cargo run -- [-timeout 2s] [-p 8] hote:port [hote:port...]
//
// Ce fichier est fourni complet : n'y touche pas, le travail est dans
// src/lib.rs. main ne contient aucune logique : il lit les options, construit les cibles,
// appelle sonder_tous et affiche. Les options sont lues à la main, sans
// crate : clap arrive au chapitre 12.

use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use pinger::{Cible, TIMEOUT_DEFAUT, duree_courte, format_tableau, parse_cible, sonder_tous};

const USAGE: &str = "usage : pinger [-timeout 2s] [-p 8] hote:port [hote:port...]";

/// Lit une durée à la Go : "500ms", "2s", "1m". Rend None si ça ne se lit pas.
fn parse_duree(s: &str) -> Option<Duration> {
    // L'ordre compte : "ms" avant "s", sinon "500ms" serait lu "500m" + "s".
    if let Some(n) = s.strip_suffix("ms") {
        n.parse().ok().map(Duration::from_millis)
    } else if let Some(n) = s.strip_suffix('s') {
        n.parse().ok().map(Duration::from_secs)
    } else if let Some(n) = s.strip_suffix('m') {
        n.parse::<u64>().ok().map(|m| Duration::from_secs(m * 60))
    } else {
        None
    }
}

/// Une erreur d'usage : message et usage sur stderr, code 2 comme le flag de
/// Go et comme clap.
fn usage(message: &str) -> ExitCode {
    if !message.is_empty() {
        eprintln!("pinger : {message}");
    }
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

// main peut rendre un ExitCode : c'est la façon propre de choisir le code de
// sortie sans appeler process::exit, qui ne lancerait pas les Drop.
fn main() -> ExitCode {
    let mut timeout = TIMEOUT_DEFAUT;
    let mut parallelisme = 8;
    let mut textes = Vec::new();

    // skip(1) : le premier argument est le nom du programme. On garde
    // l'itérateur dans une variable pour pouvoir appeler next() dans la
    // boucle et consommer la valeur d'une option.
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-timeout" | "--timeout" => match args.next().as_deref().and_then(parse_duree) {
                Some(d) => timeout = d,
                None => return usage("-timeout attend une durée (500ms, 2s, 1m)"),
            },
            "-p" => match args.next().and_then(|v| v.parse().ok()) {
                Some(n) => parallelisme = n,
                None => return usage("-p attend un entier"),
            },
            "-h" | "--help" => return usage(""),
            _ => textes.push(arg),
        }
    }
    if textes.is_empty() {
        return usage("aucune cible");
    }

    // Chaque argument restant est une cible "hote:port".
    let mut cibles: Vec<Cible> = Vec::new();
    for t in &textes {
        match parse_cible(t) {
            Ok(mut c) => {
                c.timeout = Some(timeout);
                cibles.push(c);
            }
            Err(e) => return usage(&format!("cible {t:?} : {e}")),
        }
    }

    // Le drapeau d'arrêt. Personne ne le lève ici : un gestionnaire de
    // Ctrl-C (crate ctrlc, voir « Pour aller plus loin ») le ferait.
    let arret = Arc::new(AtomicBool::new(false));

    let debut = Instant::now();
    let res = sonder_tous(&cibles, parallelisme, arret);
    print!("{}", format_tableau(&res)); // le résultat : stdout
    eprintln!("{} cibles en {}", res.len(), duree_courte(debut.elapsed())); // le commentaire : stderr

    // Code 1 dès qu'une cible est injoignable, pour que
    // `pinger ... && deploy` fasse ce qu'on attend.
    if res.iter().all(|r| r.ok) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
