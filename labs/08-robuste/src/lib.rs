// Labo 08 : un lecteur de configuration qui dit toujours pourquoi il refuse.
// Rôle du fichier : la bibliothèque. Elle ne panique jamais sur une entrée,
// n'affiche rien et ne quitte jamais le programme : elle rend une Config ou
// une ErreurConfig, et c'est l'appelant (main.rs, ou un test) qui décide.
// Lancé par : cargo test   (tout est rouge au départ, mais tout compile)
//
// Format accepté :
//     # commentaire
//     port = 5432                  obligatoire, 1 à 65535
//     timeout = 3                  en secondes, 5 par défaut
//     hotes = db-01, db-02         obligatoire, au moins un hôte
//     verbeux = true               true ou false, false par défaut
// Une clé répétée : la dernière valeur gagne.

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;

/// La configuration, une fois lue et validée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub port: u16,
    pub timeout: Duration,
    pub hotes: Vec<String>,
    pub verbeux: bool,
}

// La cause d'une valeur invalide : n'importe quelle erreur (ParseIntError,
// ParseBoolError, ou un simple texte). Send + Sync pour qu'anyhow l'accepte.
type Cause = Box<dyn Error + Send + Sync>;

/// Tout ce qui peut empêcher de lire une configuration.
#[derive(Debug, Error)]
pub enum ErreurConfig {
    // TODO 1 : les cinq messages. Remplace chaque "TODO" par le texte exact
    // attendu par les tests (cherche to_string() dans tests/config.rs). Les champs
    // s'insèrent par leur nom ({numero}, {cle}) ou leur position ({0}) ; un
    // PathBuf n'est pas Display, écris #[error("lecture de {}", chemin.display())].
    // Ne répète pas la cause dans le message : c'est #[source] qui la porte.
    #[error("TODO")]
    Io {
        chemin: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("TODO")]
    LigneInvalide { numero: usize, contenu: String },

    #[error("TODO")]
    CleInconnue { numero: usize, cle: String },

    #[error("TODO")]
    ValeurInvalide {
        numero: usize,
        cle: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("TODO")]
    CleManquante(&'static str),
}

// Les deux fonctions d'aide ne sont appelées que par parser, qui n'est encore
// qu'un todo!() : sans ces lignes, le compilateur signale du code mort.
// Retire les deux #[allow(dead_code)] quand parser les appelle.

/// Un port : un u16 non nul.
#[allow(dead_code)]
fn valeur_port(texte: &str) -> Result<u16, Cause> {
    // TODO 2 : `let port: u16 = texte.parse()?;` (le ? met la ParseIntError
    // en boîte tout seul), puis refuse 0 avec
    // `return Err("le port 0 est réservé".into());`.
    let _ = texte;
    todo!("TODO 2")
}

/// Une liste d'hôtes séparés par des virgules ; au moins un.
#[allow(dead_code)]
fn valeur_hotes(texte: &str) -> Result<Vec<String>, Cause> {
    // TODO 3 : split(','), map(str::trim), filter des morceaux vides,
    // map(String::from), collect. Si la liste est vide :
    // Err("la liste d'hôtes est vide".into()).
    let _ = texte;
    todo!("TODO 3")
}

/// Analyse le texte d'une configuration. S'arrête à la première erreur.
pub fn parser(texte: &str) -> Result<Config, ErreurConfig> {
    // TODO 4 : quatre variables Option (port, timeout, hotes, verbeux), puis
    // une boucle sur texte.lines().enumerate() ; numero = i + 1. Ligne
    // trim()ée vide ou commençant par '#' : continue. split_once('=') rend
    // un Option : .ok_or_else(|| ErreurConfig::LigneInvalide { ... })?.
    //
    // TODO 5 : un match sur la clé (trim()ée), un bras par clé connue, et
    // CleInconnue pour le reste. Pour une valeur invalide, une fermeture
    // `let invalide = |source: Cause| ErreurConfig::ValeurInvalide { numero, cle: cle.to_string(), source };`
    // et `valeur_port(valeur).map_err(invalide)?`. timeout : parse::<u64>()
    // puis Duration::from_secs ; verbeux : parse::<bool>(). Leurs erreurs se
    // mettent en boîte avec map_err(|e| invalide(Box::new(e))).
    //
    // TODO 6 : la Config finale. port et hotes : ok_or(ErreurConfig::CleManquante("port"))? ;
    // timeout et verbeux : unwrap_or(la valeur par défaut).
    let _ = texte;
    todo!("TODO 4, 5 et 6")
}

/// Lit et analyse le fichier `chemin`.
pub fn charger(chemin: &Path) -> Result<Config, ErreurConfig> {
    // TODO 7 : std::fs::read_to_string(chemin), puis
    // .map_err(|source| ErreurConfig::Io { chemin: chemin.to_path_buf(), source })?
    // (pas de #[from] possible : il faut le chemin), et enfin parser(&texte).
    let _ = chemin;
    todo!("TODO 7")
}
