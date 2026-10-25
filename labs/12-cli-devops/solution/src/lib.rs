// Labo 12, solution : la bibliothèque de dsk, toutes les fonctions testables.
// Lancé par : (cd solution && cargo test)   depuis labs/12-cli-devops
//
// Aucune fonction d'ici n'écrit sur stdout ni ne lit les arguments : elles
// prennent des chemins, des lecteurs, des tranches, et rendent des valeurs.
// main.rs (la CLI clap) décide de l'affichage. C'est ce découpage qui rend
// un outil en ligne de commande testable sans lancer de processus.

//! Petits outils DevOps : les plus gros fichiers d'un dossier, le filtrage
//! d'un journal, les variables d'environnement, le lancement d'une commande
//! avec un rapport JSON.

#![warn(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

/// Un fichier et sa taille en octets.
// Serialize : serde_json sait écrire un PathBuf (en chaîne, s'il est en
// UTF-8 valide). Deserialize pour pouvoir relire un rapport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fichier {
    /// Le chemin, tel que trouvé en parcourant le dossier.
    pub chemin: PathBuf,
    /// La taille en octets.
    pub taille: u64,
}

/// Une taille pour un humain, en puissances de 1024 comme `du -h` :
/// `"512 o"`, `"1.5 Kio"`, `"3.0 Gio"`.
pub fn taille_lisible(octets: u64) -> String {
    const UNITES: [&str; 5] = ["Kio", "Mio", "Gio", "Tio", "Pio"];
    if octets < 1024 {
        return format!("{octets} o");
    }
    // f64 pour la division avec décimales : la perte de précision au-delà de
    // 2^53 octets (8 Pio) ne change rien à un affichage à un chiffre après
    // la virgule.
    let mut valeur = octets as f64 / 1024.0;
    let mut unite = 0;
    while valeur >= 1024.0 && unite < UNITES.len() - 1 {
        valeur /= 1024.0;
        unite += 1;
    }
    format!("{valeur:.1} {}", UNITES[unite])
}

/// Les `n` plus gros fichiers sous `dir`, récursivement, du plus gros au
/// plus petit (à taille égale, par chemin). Les liens symboliques ne sont
/// pas suivis : un lien vers `/` ne fait pas parcourir tout le disque.
pub fn plus_gros(dir: &Path, n: usize) -> anyhow::Result<Vec<Fichier>> {
    let mut tous = Vec::new();
    // with_context ajoute le chemin au message : « lire le dossier
    // /nulle/part » suivi de la cause système, au lieu d'un « No such file
    // or directory » qui ne dit pas quel fichier.
    parcourir(dir, &mut tous).with_context(|| format!("parcourir {}", dir.display()))?;
    // Tri décroissant par taille : on compare b à a. then_with départage
    // par chemin, pour un résultat stable d'un lancement à l'autre.
    tous.sort_by(|a, b| {
        b.taille
            .cmp(&a.taille)
            .then_with(|| a.chemin.cmp(&b.chemin))
    });
    tous.truncate(n);
    Ok(tous)
}

/// Le parcours récursif. Privé : plus_gros est l'API, ceci un détail.
/// Il rend une io::Error brute ; plus_gros y ajoute le contexte.
fn parcourir(dir: &Path, tous: &mut Vec<Fichier>) -> io::Result<()> {
    // read_dir rend un itérateur de Result<DirEntry> : chaque entrée peut
    // échouer à part (fichier supprimé pendant le parcours), d'où le `?`.
    for entree in fs::read_dir(dir)? {
        let entree = entree?;
        // file_type() ne suit pas les liens : un lien symbolique est vu
        // comme un lien, ni fichier ni dossier, et on l'ignore.
        let genre = entree.file_type()?;
        let chemin = entree.path();
        if genre.is_dir() {
            parcourir(&chemin, tous)?;
        } else if genre.is_file() {
            let taille = entree.metadata()?.len();
            tous.push(Fichier { chemin, taille });
        }
    }
    Ok(())
}

/// Le niveau d'une ligne de journal, du moins grave au plus grave.
// L'ordre des variantes donne l'ordre de Ord : Debug < Info < Warn < Error,
// ce qui permet `niveau >= niveau_min`. rename_all écrit "ERROR" en JSON
// au lieu de "Error", comme dans le journal lui-même.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Niveau {
    /// Détail de mise au point.
    Debug,
    /// Information normale.
    Info,
    /// Quelque chose d'anormal mais pas bloquant.
    Warn,
    /// Une erreur.
    Error,
}

impl fmt::Display for Niveau {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let texte = match self {
            Niveau::Debug => "DEBUG",
            Niveau::Info => "INFO",
            Niveau::Warn => "WARN",
            Niveau::Error => "ERROR",
        };
        // f.pad et non write! : pad respecte la largeur demandée par
        // l'appelant ({:<5}), write! l'ignorerait.
        f.pad(texte)
    }
}

impl FromStr for Niveau {
    // Une String suffit comme erreur : clap sait l'afficher (il accepte tout
    // ce qui se convertit en Box<dyn Error>), et main n'en fait rien d'autre.
    type Err = String;

    /// Insensible à la casse : `error`, `ERROR` et `Error` se valent.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "DEBUG" => Ok(Niveau::Debug),
            "INFO" => Ok(Niveau::Info),
            // WARNING est l'orthographe de Python et de syslog : on l'accepte.
            "WARN" | "WARNING" => Ok(Niveau::Warn),
            "ERROR" => Ok(Niveau::Error),
            _ => Err(format!("niveau {s:?} inconnu (DEBUG, INFO, WARN ou ERROR)")),
        }
    }
}

/// Une ligne de journal lue : `2026-10-25 01:02:03 ERROR message`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entree {
    /// La date et l'heure, telles qu'écrites (`2026-10-25 01:02:03`).
    pub horodatage: String,
    /// Le niveau.
    pub niveau: Niveau,
    /// Le reste de la ligne.
    pub message: String,
}

/// Lit une ligne de journal, ou None si elle n'a pas la bonne forme.
fn parser_ligne(ligne: &str) -> Option<Entree> {
    // splitn(4, ' ') : date, heure, niveau, puis TOUT le reste, espaces
    // compris, dans le quatrième morceau.
    let mut morceaux = ligne.splitn(4, ' ');
    let date = morceaux.next()?;
    let heure = morceaux.next()?;
    // .ok()? : un niveau illisible fait de la ligne une ligne à ignorer.
    let niveau = morceaux.next()?.parse().ok()?;
    let message = morceaux.next().unwrap_or("").trim_end();
    // Une vérification minimale de la forme : 10 caractères de date, 8
    // d'heure. Assez pour écarter une ligne de pile d'exception.
    if date.len() != 10 || heure.len() != 8 {
        return None;
    }
    Some(Entree {
        horodatage: format!("{date} {heure}"),
        niveau,
        message: message.to_string(),
    })
}

/// Les entrées d'un journal dont le niveau est au moins `niveau_min`. Les
/// lignes mal formées sont ignorées, comme le ferait un `grep`.
///
/// `impl BufRead` : un fichier dans un `BufReader`, `stdin().lock()`, ou un
/// `&[u8]` dans les tests, sans changer une ligne.
pub fn parser_logs(lecteur: impl BufRead, niveau_min: Niveau) -> Vec<Entree> {
    lecteur
        .lines()
        // lines() rend des io::Result<String> : on s'arrête à la première
        // erreur de lecture (UTF-8 invalide, disque arraché). map_while
        // continue tant que la fermeture rend Some.
        .map_while(Result::ok)
        // filter_map : parse et filtre d'un coup, les None disparaissent.
        .filter_map(|ligne| parser_ligne(&ligne))
        .filter(|e| e.niveau >= niveau_min)
        .collect()
}

/// Le nombre d'entrées par niveau. BTreeMap plutôt que HashMap : les clés
/// sortent triées, donc l'affichage et le JSON sont stables.
pub fn bilan(entrees: &[Entree]) -> BTreeMap<Niveau, usize> {
    let mut compte = BTreeMap::new();
    for e in entrees {
        // entry().or_insert(0) puis += 1 : le compteur de mots du chapitre 5.
        *compte.entry(e.niveau).or_insert(0) += 1;
    }
    compte
}

/// Garde les paires dont le nom commence par `prefixe`, triées par nom.
/// Séparée de [`variables`] pour être testée avec une fausse liste, sans
/// toucher au vrai environnement du processus.
pub fn filtrer_variables(
    vars: impl IntoIterator<Item = (String, String)>,
    prefixe: &str,
) -> Vec<(String, String)> {
    let mut gardees: Vec<_> = vars
        .into_iter()
        .filter(|(nom, _)| nom.starts_with(prefixe))
        .collect();
    // Les tuples se comparent champ par champ : tri par nom, puis valeur.
    gardees.sort();
    gardees
}

/// Les variables d'environnement dont le nom commence par `prefixe`, triées.
pub fn variables(prefixe: &str) -> Vec<(String, String)> {
    // env::vars() panique sur une variable qui n'est pas de l'UTF-8 ;
    // vars_os() ne panique jamais. On prend vars_os et on écarte ce qui ne
    // se lit pas, plutôt que de planter sur l'environnement d'un autre.
    let lisibles = std::env::vars_os()
        .filter_map(|(nom, valeur)| Some((nom.into_string().ok()?, valeur.into_string().ok()?)));
    filtrer_variables(lisibles, prefixe)
}

/// Le rapport d'une commande lancée par [`lancer`], prêt pour serde_json.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rapport {
    /// La commande et ses arguments, séparés par des espaces.
    pub commande: String,
    /// Le code de sortie ; `None` si le processus a été tué par un signal.
    pub code: Option<i32>,
    /// La durée, en millisecondes.
    pub duree_ms: u64,
    /// Ce que la commande a écrit sur sa sortie standard.
    pub stdout: String,
    /// Ce qu'elle a écrit sur sa sortie d'erreur.
    pub stderr: String,
}

/// Lance `commande[0]` avec les arguments `commande[1..]`, attend sa fin,
/// et rend le rapport. Une commande introuvable est une erreur ; une
/// commande qui échoue (code non nul) ne l'est pas : c'est un rapport.
pub fn lancer(commande: &[String]) -> anyhow::Result<Rapport> {
    // Un motif de tranche : le premier élément et le reste, ou rien.
    let [programme, arguments @ ..] = commande else {
        bail!("aucune commande à lancer");
    };
    let debut = Instant::now();
    // Command ne passe pas par un shell : pas d'expansion de *, pas
    // d'injection possible par un argument qui contiendrait « ; rm -rf ».
    // output() capture stdout et stderr et attend la fin du processus.
    let sortie = Command::new(programme)
        .args(arguments)
        .output()
        .with_context(|| format!("lancer {programme:?}"))?;
    Ok(Rapport {
        commande: commande.join(" "),
        // status.code() est None quand un signal a tué le processus (Unix).
        code: sortie.status.code(),
        // as_millis rend un u128 ; try_from vers u64 ne peut échouer qu'au
        // bout de 584 millions d'années, unwrap_or(u64::MAX) le dit sans
        // paniquer.
        duree_ms: u64::try_from(debut.elapsed().as_millis()).unwrap_or(u64::MAX),
        // Les sorties sont des octets : lossy remplace ce qui n'est pas de
        // l'UTF-8 au lieu d'échouer. into_owned rend une String possédée.
        stdout: String::from_utf8_lossy(&sortie.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&sortie.stderr).into_owned(),
    })
}
