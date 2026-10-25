// Labo 16, solution : api.rs, les routes HTTP du service.
//
// Quatre routes, quatre handlers axum. Ils ne font que lire : l'état est
// écrit par la boucle de sondage (lib.rs), sous les mêmes verrous. Aucun
// handler ne garde un verrou pendant un .await : il n'y a d'ailleurs aucun
// .await dans ces handlers, tout est en mémoire ou dans un SQLite local.

use std::fmt::Write as _; // writeln! dans une String
use std::sync::Arc; // l'état partagé
use std::sync::atomic::Ordering; // lire le compteur de tours

use axum::extract::rejection::QueryRejection; // un ?n=abc mal formé
use axum::extract::{Query, State}; // les extracteurs utilisés
use axum::http::{StatusCode, header}; // statuts et Content-Type
use axum::response::{IntoResponse, Response}; // ce qu'un handler renvoie
use axum::routing::get; // une seule méthode ici : GET
use axum::{Json, Router};
use serde::{Deserialize, Serialize}; // les paramètres de requête et les réponses
use serde_json::json; // les petits corps ad hoc

use crate::Etat; // l'état partagé, défini dans lib.rs
use crate::sonde::{Mesure, maintenant};

/// Le nombre de mesures rendues par /api/historique sans `n`, et le maximum.
const HISTORIQUE_DEFAUT: usize = 20;
const HISTORIQUE_MAX: usize = 1_000;

/// Construit le routeur. L'état est passé en paramètre pour que les tests
/// gardent une main dessus et y injectent des mesures.
pub fn routeur(etat: Arc<Etat>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/status", get(status))
        .route("/api/historique", get(historique))
        .route("/metrics", get(metrics))
        .with_state(etat)
}

/// Les erreurs de l'API, en JSON comme au labo 13.
#[derive(Debug)]
pub enum ErreurApi {
    CibleInconnue(String),
    RequeteInvalide(String),
    Interne(String),
}

impl IntoResponse for ErreurApi {
    fn into_response(self) -> Response {
        let (statut, message) = match self {
            ErreurApi::CibleInconnue(nom) => {
                (StatusCode::NOT_FOUND, format!("cible {nom:?} inconnue"))
            }
            ErreurApi::RequeteInvalide(m) => (StatusCode::BAD_REQUEST, m),
            // Le détail d'une erreur interne va au journal, pas au client.
            ErreurApi::Interne(m) => {
                eprintln!("erreur interne : {m}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "erreur interne".to_string(),
                )
            }
        };
        (statut, Json(json!({ "erreur": message }))).into_response()
    }
}

/// Une erreur du dépôt (anyhow) devient un 500 : `?` l'appelle tout seul.
impl From<anyhow::Error> for ErreurApi {
    fn from(e: anyhow::Error) -> Self {
        // {e:#} : la chaîne des causes sur une ligne, pour le journal.
        ErreurApi::Interne(format!("{e:#}"))
    }
}

/// `GET /healthz` : le processus est vivant et répond.
async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "statut": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

/// Une ligne de /api/status. Les champs de la dernière mesure sont des
/// Option : avant le premier tour, on connaît la cible mais pas son état, et
/// `null` le dit mieux qu'un faux `false`.
#[derive(Debug, Serialize)]
pub struct StatutCible {
    pub nom: String,
    pub adresse: String,
    pub ok: Option<bool>,
    pub duree_ms: Option<u32>,
    pub erreur: Option<String>,
    pub instant: Option<i64>,
    /// Entre 0.0 et 1.0, sur les dernières 24 heures ; `null` sans mesure.
    pub disponibilite_24h: Option<f64>,
}

/// Le corps de /api/status.
#[derive(Debug, Serialize)]
pub struct Statut {
    pub version: &'static str,
    pub tours: u64,
    pub cibles: Vec<StatutCible>,
}

/// `GET /api/status` : l'état courant de chaque cible, dans l'ordre de la
/// configuration, avec sa disponibilité sur 24 heures.
async fn status(State(etat): State<Arc<Etat>>) -> Result<Json<Statut>, ErreurApi> {
    let depuis = maintenant() - 86_400;
    // Les deux verrous sont pris l'un après l'autre, chacun dans son propre
    // bloc, jamais l'un dans l'autre : impossible de s'interbloquer avec la
    // boucle de sondage, qui les prend aussi l'un après l'autre.
    let derniers = etat.derniers.read().expect("verrou empoisonné").clone();
    let depot = etat.depot.lock().expect("verrou empoisonné");
    let mut cibles = Vec::with_capacity(etat.config.cibles.len());
    for c in &etat.config.cibles {
        let m = derniers.get(&c.nom);
        cibles.push(StatutCible {
            nom: c.nom.clone(),
            adresse: c.adresse.clone(),
            ok: m.map(|m| m.ok),
            duree_ms: m.map(|m| m.duree_ms),
            erreur: m.and_then(|m| m.erreur.clone()),
            instant: m.map(|m| m.instant),
            // Le ? passe par From<anyhow::Error> : une base cassée est un 500.
            disponibilite_24h: depot.taux_disponibilite(&c.nom, depuis)?,
        });
    }
    Ok(Json(Statut {
        version: env!("CARGO_PKG_VERSION"),
        tours: etat.tours.load(Ordering::Relaxed),
        cibles,
    }))
}

/// Les paramètres de /api/historique. serde lit la chaîne de requête : un
/// paramètre absent donne None, un `n=abc` fait échouer l'extraction.
#[derive(Debug, Deserialize)]
pub struct ParamsHistorique {
    pub cible: Option<String>,
    pub n: Option<usize>,
}

/// `GET /api/historique?cible=…&n=…` : les dernières mesures, la plus
/// récente d'abord. 404 pour une cible absente de la configuration.
async fn historique(
    State(etat): State<Arc<Etat>>,
    params: Result<Query<ParamsHistorique>, QueryRejection>,
) -> Result<Json<Vec<Mesure>>, ErreurApi> {
    let Query(params) = params.map_err(|r| ErreurApi::RequeteInvalide(r.body_text()))?;
    // Une « let chain » (édition 2024) : le if let et la condition dans le
    // même if, au lieu de deux if imbriqués.
    if let Some(nom) = &params.cible
        && !etat.config.cibles.iter().any(|c| &c.nom == nom)
    {
        return Err(ErreurApi::CibleInconnue(nom.clone()));
    }
    let n = params.n.unwrap_or(HISTORIQUE_DEFAUT).min(HISTORIQUE_MAX);
    let mesures = etat
        .depot
        .lock()
        .expect("verrou empoisonné")
        .dernieres(params.cible.as_deref(), n)?;
    Ok(Json(mesures))
}

/// `GET /metrics` : le format texte que Prometheus vient lire.
async fn metrics(State(etat): State<Arc<Etat>>) -> impl IntoResponse {
    // Les dernières mesures dans l'ordre de la configuration : une sortie
    // stable d'un scrape à l'autre, que les tests comparent à l'octet près.
    let mesures: Vec<Mesure> = {
        let derniers = etat.derniers.read().expect("verrou empoisonné");
        etat.config
            .cibles
            .iter()
            .filter_map(|c| derniers.get(&c.nom).cloned())
            .collect()
    };
    let corps = format_prometheus(&mesures, etat.tours.load(Ordering::Relaxed));
    // Un tuple (en-têtes, corps) implémente IntoResponse : statut 200.
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        corps,
    )
}

/// Échappe une valeur de label Prometheus : la barre oblique inverse, le
/// guillemet et le saut de ligne. Un nom de cible bizarre ne doit pas casser
/// le scrape.
#[must_use]
pub fn echapper_label(valeur: &str) -> String {
    valeur
        .replace('\\', r"\\") // d'abord la barre oblique, sinon on doublerait celles qu'on ajoute
        .replace('"', r#"\""#)
        .replace('\n', r"\n")
}

/// Le format d'exposition de Prometheus, écrit à la main : pour chaque
/// métrique une ligne `# HELP`, une ligne `# TYPE`, puis une ligne par série.
///
/// ```text
/// # HELP sondes_up 1 si la cible a répondu à la dernière sonde, 0 sinon.
/// # TYPE sondes_up gauge
/// sondes_up{cible="web"} 1
/// ```
#[must_use]
pub fn format_prometheus(mesures: &[Mesure], tours: u64) -> String {
    let mut s = String::new();
    // writeln! dans une String ne peut pas échouer : on ignore le Result.
    let _ = writeln!(
        s,
        "# HELP sondes_up 1 si la cible a répondu à la dernière sonde, 0 sinon."
    );
    let _ = writeln!(s, "# TYPE sondes_up gauge");
    for m in mesures {
        // u8::from(bool) : 1 ou 0, Prometheus n'a pas de booléens.
        let _ = writeln!(
            s,
            "sondes_up{{cible=\"{}\"}} {}",
            echapper_label(&m.cible),
            u8::from(m.ok)
        );
    }
    let _ = writeln!(
        s,
        "# HELP sondes_duree_ms Durée de la dernière sonde, en millisecondes."
    );
    let _ = writeln!(s, "# TYPE sondes_duree_ms gauge");
    for m in mesures {
        let _ = writeln!(
            s,
            "sondes_duree_ms{{cible=\"{}\"}} {}",
            echapper_label(&m.cible),
            m.duree_ms
        );
    }
    let _ = writeln!(
        s,
        "# HELP sondes_tours_total Tours de sondage effectués depuis le démarrage."
    );
    let _ = writeln!(s, "# TYPE sondes_tours_total counter");
    let _ = writeln!(s, "sondes_tours_total {tours}");
    s
}
