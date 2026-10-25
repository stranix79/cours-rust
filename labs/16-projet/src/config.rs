// Labo 16 : config.rs, le fichier sondes.toml.
//
// Deux étapes, comme dans tout bon chargeur de configuration : serde lit le
// TOML dans des structs « brutes » qui ressemblent au fichier (des chaînes
// pour les durées), puis valider() les transforme en Config, le type que le
// reste du programme utilise (des Duration, des règles vérifiées). Le reste
// du programme ne voit jamais une config invalide : elle n'a pas de type.

use std::collections::HashSet; // repérer les noms de cibles en double
use std::path::Path; // le chemin du fichier
use std::time::Duration; // ce que deviennent "10s" et "500ms"

use anyhow::{Context, bail}; // contexte sur les erreurs, bail! pour en créer
use serde::Deserialize; // serde lit le TOML dans nos structs

use crate::sonde::Cible; // la config fabrique les cibles que sonde.rs sonde

/// La configuration validée, prête à l'emploi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// L'adresse d'écoute de l'API (`127.0.0.1:8080` par défaut).
    pub ecoute: String,
    /// Le fichier SQLite des mesures (`sondes.db` par défaut, `:memory:`
    /// pour une base jetable).
    pub base: String,
    /// Le temps entre deux tours de sondage.
    pub intervalle: Duration,
    /// Le nombre maximal de sondes en même temps (le worker pool).
    pub parallelisme: usize,
    /// Les cibles, dans l'ordre du fichier.
    pub cibles: Vec<Cible>,
}

/// Le fichier tel qu'il est écrit. deny_unknown_fields : une faute de frappe
/// (`intervale = "5s"`) est une erreur, pas un réglage ignoré en silence qui
/// laisse la valeur par défaut. `#[serde(default = ...)]` donne la valeur
/// quand la clé manque.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FichierBrut {
    #[serde(default = "defaut_ecoute")]
    ecoute: String,
    #[serde(default = "defaut_base")]
    base: String,
    #[serde(default = "defaut_intervalle")]
    intervalle: String,
    #[serde(default = "defaut_parallelisme")]
    parallelisme: usize,
    // Dans le TOML, chaque cible est une table [[cible]] : le nom de la clé
    // est au singulier, comme on l'écrit dans le fichier. rename fait le lien
    // avec notre champ au pluriel. default : un fichier sans cible donne un
    // Vec vide, que valider() refuse avec un message clair.
    #[serde(rename = "cible", default)]
    cibles: Vec<CibleBrute>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CibleBrute {
    nom: String,
    adresse: String,
    #[serde(default = "defaut_timeout")]
    timeout: String,
}

// serde veut des fonctions pour les valeurs par défaut, pas des constantes.
fn defaut_ecoute() -> String {
    "127.0.0.1:8080".to_string()
}
fn defaut_base() -> String {
    "sondes.db".to_string()
}
fn defaut_intervalle() -> String {
    "10s".to_string()
}
fn defaut_parallelisme() -> usize {
    8
}
fn defaut_timeout() -> String {
    "1s".to_string()
}

/// Lit une durée écrite comme un humain l'écrit : `500ms`, `10s`, `2m`, `1h`.
/// Un nombre entier suivi d'une unité, sans espace. Le `time.ParseDuration`
/// de Go, en plus petit.
///
/// # Errors
/// Nombre absent ou invalide, unité inconnue, ou durée nulle.
pub fn parse_duree(texte: &str) -> Result<Duration, String> {
    // TODO 1 : trim(), puis find(|c: char| !c.is_ascii_digit()) donne la
    // fin du nombre (None : pas d'unité, erreur). split_at coupe en (nombre,
    // unité) ; parse::<u64>() le nombre (vide ou invalide : erreur) ; un
    // match sur l'unité : "ms" -> Duration::from_millis, "s" -> from_secs,
    // "m" et "h" -> from_secs(n.saturating_mul(60 ou 3_600)), autre ->
    // Err(format!(...)) en citant l'unité avec {autre:?}. Une durée nulle est
    // refusée. Les erreurs sont des String : Err(format!("durée {texte:?} : ...")).
    todo!("TODO 1")
}

/// Lit et valide une configuration depuis le texte d'un fichier TOML.
///
/// # Errors
/// TOML invalide, clé inconnue, durée illisible, aucune cible, nom en
/// double, adresse sans port, timeout plus long que l'intervalle.
pub fn parser(texte: &str) -> anyhow::Result<Config> {
    // toml::from_str fait tout le travail de lecture ; son erreur cite la
    // ligne et la colonne fautives, on la garde telle quelle sous notre
    // contexte.
    let brut: FichierBrut = toml::from_str(texte).context("sondes.toml illisible")?;
    valider(brut)
}

/// Lit le fichier puis appelle [`parser`].
///
/// # Errors
/// Fichier introuvable ou illisible, puis les erreurs de [`parser`].
pub fn charger(chemin: impl AsRef<Path>) -> anyhow::Result<Config> {
    let chemin = chemin.as_ref();
    let texte = std::fs::read_to_string(chemin)
        .with_context(|| format!("lecture de {}", chemin.display()))?;
    parser(&texte).with_context(|| format!("configuration {}", chemin.display()))
}

/// Les règles, dans l'ordre où on les vérifie. On s'arrête à la première
/// erreur : un message précis vaut mieux qu'une liste de dix.
fn valider(brut: FichierBrut) -> anyhow::Result<Config> {
    // map_err : notre erreur est une String, anyhow::anyhow! en fait une
    // anyhow::Error ; le ? la remonte.
    let intervalle =
        parse_duree(&brut.intervalle).map_err(|e| anyhow::anyhow!("intervalle : {e}"))?;
    if brut.parallelisme == 0 {
        bail!("parallelisme : au moins 1");
    }
    if brut.cibles.is_empty() {
        bail!("aucune cible : ajoute au moins une table [[cible]]");
    }
    let mut vus = HashSet::new();
    let mut cibles = Vec::with_capacity(brut.cibles.len());
    for c in brut.cibles {
        let nom = c.nom.trim();
        if nom.is_empty() {
            bail!("une cible a un nom vide");
        }
        // insert rend false si la valeur était déjà dans l'ensemble.
        if !vus.insert(nom.to_string()) {
            bail!("cible {nom:?} en double");
        }
        // rsplit_once : le DERNIER deux-points sépare le port, ce qui laisse
        // passer "[::1]:22". Le port doit être un u16 non nul.
        let port_valide = c
            .adresse
            .rsplit_once(':')
            .and_then(|(_, port)| port.parse::<u16>().ok())
            .is_some_and(|port| port != 0);
        if !port_valide {
            bail!(
                "cible {nom:?} : adresse {:?} sans port valide (attendu hote:port)",
                c.adresse
            );
        }
        let timeout = parse_duree(&c.timeout)
            .map_err(|e| anyhow::anyhow!("cible {nom:?} : timeout : {e}"))?;
        // Une sonde qui dure plus longtemps que l'intervalle chevaucherait
        // le tour suivant : on le refuse dès le chargement.
        if timeout > intervalle {
            bail!(
                "cible {nom:?} : timeout {} plus long que l'intervalle {}",
                c.timeout,
                brut.intervalle
            );
        }
        cibles.push(Cible {
            nom: nom.to_string(),
            adresse: c.adresse,
            timeout,
        });
    }
    Ok(Config {
        ecoute: brut.ecoute,
        base: brut.base,
        intervalle,
        parallelisme: brut.parallelisme,
        cibles,
    })
}
