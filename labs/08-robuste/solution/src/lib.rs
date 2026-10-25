// Labo 08, solution : un lecteur de configuration qui dit toujours pourquoi il refuse.
// Lancé par : (cd solution && cargo test)   depuis labs/08-robuste
//
// Rôle du fichier : la bibliothèque. Elle ne panique jamais sur une entrée,
// n'affiche rien et ne quitte jamais le programme : elle rend une Config ou
// une ErreurConfig, et c'est l'appelant (main.rs, ou un test) qui décide.
// D'où thiserror ici : l'appelant doit pouvoir tester la variante.
//
// Format accepté :
//     # commentaire
//     port = 5432                  obligatoire, 1 à 65535
//     timeout = 3                  en secondes, 5 par défaut
//     hotes = db-01, db-02         obligatoire, au moins un hôte
//     verbeux = true               true ou false, false par défaut
// Une clé répétée : la dernière valeur gagne.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;

/// La configuration, une fois lue et validée. Tous les champs ont une
/// valeur : les défauts sont appliqués par `parser`, pas par l'appelant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub port: u16,
    pub timeout: Duration,
    pub hotes: Vec<String>,
    pub verbeux: bool,
}

// La cause d'une valeur invalide : n'importe quelle erreur (ParseIntError,
// ParseBoolError, ou un simple texte). Send + Sync pour que l'erreur puisse
// changer de thread et entrer dans un anyhow::Error, qui l'exige.
type Cause = Box<dyn Error + Send + Sync>;

/// Tout ce qui peut empêcher de lire une configuration. Chaque variante porte
/// de quoi écrire un message utile : le chemin, le numéro de ligne, la clé.
#[derive(Debug, Error)]
pub enum ErreurConfig {
    // Le message ne répète pas la cause (« No such file... ») : c'est source
    // qui la porte, et l'affichage {:#} d'anyhow la mettra au bout.
    #[error("lecture de {}", chemin.display())]
    Io {
        chemin: PathBuf,
        // #[source] et pas #[from] : un From<io::Error> ne saurait pas quel
        // chemin mettre dans la variante. On la construit avec map_err.
        #[source]
        source: io::Error,
    },

    #[error("ligne {numero} : « {contenu} » n'est pas de la forme cle = valeur")]
    LigneInvalide { numero: usize, contenu: String },

    #[error("ligne {numero} : clé inconnue « {cle} »")]
    CleInconnue { numero: usize, cle: String },

    #[error("ligne {numero} : valeur invalide pour « {cle} »")]
    ValeurInvalide {
        numero: usize,
        cle: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    // &'static str : les noms de clés obligatoires sont des littéraux du
    // programme, pas des données lues ; inutile d'allouer une String.
    #[error("clé obligatoire absente : « {0} »")]
    CleManquante(&'static str),
}

/// Un port : un u16 non nul. L'erreur est une Cause : celle de parse, ou un
/// texte à nous pour le zéro.
fn valeur_port(texte: &str) -> Result<u16, Cause> {
    // Le ? convertit la ParseIntError en Box<dyn Error + Send + Sync> : la
    // std fournit un From pour toute erreur qui est Send + Sync.
    let port: u16 = texte.parse()?;
    if port == 0 {
        // Une &str se convertit aussi en Box<dyn Error> avec into().
        return Err("le port 0 est réservé".into());
    }
    Ok(port)
}

/// Une liste d'hôtes séparés par des virgules ; au moins un.
fn valeur_hotes(texte: &str) -> Result<Vec<String>, Cause> {
    // split garde les morceaux vides (« a,,b ») : on les retire après trim,
    // pour qu'une virgule en trop ne crée pas un hôte vide.
    let hotes: Vec<String> = texte
        .split(',')
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .map(String::from)
        .collect();
    if hotes.is_empty() {
        return Err("la liste d'hôtes est vide".into());
    }
    Ok(hotes)
}

/// Analyse le texte d'une configuration. S'arrête à la première erreur.
pub fn parser(texte: &str) -> Result<Config, ErreurConfig> {
    // Des Option tant qu'on lit : on ne sait qu'à la fin si une clé
    // obligatoire manque. Pas de valeur « zéro » qui voudrait dire « absent ».
    let mut port = None;
    let mut timeout = None;
    let mut hotes = None;
    let mut verbeux = None;

    // enumerate compte depuis 0, un éditeur depuis 1 : numero = i + 1.
    for (i, ligne) in texte.lines().enumerate() {
        let numero = i + 1;
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }

        // split_once coupe au premier « = » ; None si la ligne n'en a pas.
        // ok_or_else transforme ce None en notre erreur, ? la renvoie.
        let (cle, valeur) = ligne
            .split_once('=')
            .ok_or_else(|| ErreurConfig::LigneInvalide {
                numero,
                contenu: ligne.to_string(),
            })?;
        let (cle, valeur) = (cle.trim(), valeur.trim());

        // Fabrique l'erreur de valeur pour cette ligne et cette clé. Une
        // fermeture plutôt que de répéter la struct dans chaque bras du match.
        let invalide = |source: Cause| ErreurConfig::ValeurInvalide {
            numero,
            cle: cle.to_string(),
            source,
        };

        // map_err(invalide) puis ? : la Cause devient une ErreurConfig qui
        // la garde comme source, et la fonction s'arrête là.
        match cle {
            "port" => port = Some(valeur_port(valeur).map_err(invalide)?),
            "timeout" => {
                // parse::<u64> rend une ParseIntError : Box::new la met en boîte.
                let secondes: u64 = valeur.parse().map_err(|e| invalide(Box::new(e)))?;
                timeout = Some(Duration::from_secs(secondes));
            }
            "hotes" => hotes = Some(valeur_hotes(valeur).map_err(invalide)?),
            // bool implémente FromStr : « true » et « false », rien d'autre
            // (ni « oui », ni « True »). L'erreur est une ParseBoolError.
            "verbeux" => verbeux = Some(valeur.parse().map_err(|e| invalide(Box::new(e)))?),
            _ => {
                return Err(ErreurConfig::CleInconnue {
                    numero,
                    cle: cle.to_string(),
                });
            }
        }
    }

    // ok_or : None devient l'erreur CleManquante. unwrap_or : None devient
    // la valeur par défaut. Deux méthodes d'Option, aucun if.
    Ok(Config {
        port: port.ok_or(ErreurConfig::CleManquante("port"))?,
        timeout: timeout.unwrap_or(Duration::from_secs(5)),
        hotes: hotes.ok_or(ErreurConfig::CleManquante("hotes"))?,
        verbeux: verbeux.unwrap_or(false),
    })
}

/// Lit et analyse le fichier `chemin`.
pub fn charger(chemin: &Path) -> Result<Config, ErreurConfig> {
    // map_err ajoute le contexte (le chemin) que l'io::Error n'a pas : son
    // message dit « No such file or directory », sans dire quel fichier.
    let texte = fs::read_to_string(chemin).map_err(|source| ErreurConfig::Io {
        chemin: chemin.to_path_buf(),
        source,
    })?;
    parser(&texte)
}
