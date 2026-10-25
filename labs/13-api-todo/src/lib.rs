// Labo 13 : une API REST de tâches avec axum, l'état partagé et le client
// qui l'interroge.
// Lancé par : cargo test (les tests sont dans tests/api.rs), ou via main.rs :
//   cargo run -- serve --adresse 127.0.0.1:8080
//
// Les types, les imports et les signatures sont fournis. Les corps marqués
// todo!() sont à écrire, dans l'ordre des TODO. Un handler qui contient encore
// todo!() fait paniquer le test qui l'appelle : c'est le rouge attendu.

// Le squelette a des paramètres et des imports pas encore utilisés : on fait
// taire ces avertissements le temps du labo. Retire cette ligne à la fin.
#![allow(unused_variables, unused_imports, dead_code)]

use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use anyhow::Context;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;

/// Une tâche, telle que l'API la renvoie et la reçoit en JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub titre: String,
    pub fait: bool,
}

/// Le corps de `POST /todos` : seulement le titre.
#[derive(Debug, Deserialize)]
pub struct NouveauTodo {
    pub titre: String,
}

/// Le corps de `PUT /todos/{id}` : chaque champ est facultatif.
#[derive(Debug, Deserialize)]
pub struct MajTodo {
    pub titre: Option<String>,
    pub fait: Option<bool>,
}

/// L'état partagé par toutes les requêtes : un RwLock (de la std) autour du
/// Vec, et un compteur atomique pour les ids.
#[derive(Debug)]
pub struct Etat {
    pub todos: RwLock<Vec<Todo>>,
    pub prochain_id: AtomicU64,
}

impl Etat {
    /// Un état vide, déjà dans son Arc. Le premier id attribué sera 1.
    #[must_use]
    pub fn nouveau() -> Arc<Etat> {
        // TODO 1 : construis l'Etat (RwLock::new(Vec::new()), AtomicU64::new(1))
        // et mets-le dans un Arc::new(...).
        todo!("TODO 1")
    }
}

/// Les erreurs de l'API.
#[derive(Debug)]
pub enum ErreurApi {
    TitreVide,
    Introuvable(u64),
    RequeteInvalide(String),
}

impl IntoResponse for ErreurApi {
    fn into_response(self) -> Response {
        // TODO 2 : un match sur self qui donne (statut, message) :
        //   TitreVide          -> 400, "le titre est obligatoire"
        //   Introuvable(id)    -> 404, format!("todo {id} introuvable")
        //   RequeteInvalide(m) -> 400, m
        // puis renvoie (statut, Json(json!({ "erreur": message }))).into_response().
        todo!("TODO 2")
    }
}

// Ces deux conversions sont fournies : elles permettent d'écrire `corps?` et
// `id?` dans les handlers quand l'extraction a échoué (JSON invalide, id qui
// n'est pas un nombre). Lis-les, le chapitre 13 les explique.
impl From<JsonRejection> for ErreurApi {
    fn from(rejet: JsonRejection) -> Self {
        ErreurApi::RequeteInvalide(format!("JSON invalide : {}", rejet.body_text()))
    }
}

impl From<PathRejection> for ErreurApi {
    fn from(rejet: PathRejection) -> Self {
        ErreurApi::RequeteInvalide(format!("id invalide : {}", rejet.body_text()))
    }
}

/// Construit le routeur de l'API.
pub fn routeur(etat: Arc<Etat>) -> Router {
    // TODO 3 : ajoute les deux routes manquantes :
    //   .route("/todos", get(lister).post(creer))
    //   .route("/todos/{id}", get(lire).put(modifier).delete(supprimer))
    // (axum 0.8 écrit les paramètres {id}, pas :id).
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(etat)
}

/// `GET /healthz` : fourni, c'est le modèle des autres.
async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "statut": "ok" }))
}

/// `GET /todos` : la liste, dans l'ordre de création.
async fn lister(State(etat): State<Arc<Etat>>) -> Json<Vec<Todo>> {
    // TODO 4 : etat.todos.read().expect("verrou empoisonné").clone(), dans un Json.
    todo!("TODO 4")
}

/// `POST /todos` : 201 et la tâche créée, 400 si le JSON ou le titre ne va pas.
async fn creer(
    State(etat): State<Arc<Etat>>,
    corps: Result<Json<NouveauTodo>, JsonRejection>,
) -> Result<(StatusCode, Json<Todo>), ErreurApi> {
    // TODO 5 : let Json(nouveau) = corps?; puis le titre sans espaces autour
    // (trim) ; vide -> Err(ErreurApi::TitreVide). L'id vient de
    // etat.prochain_id.fetch_add(1, Ordering::Relaxed). Pousse la tâche dans
    // etat.todos.write() et renvoie Ok((StatusCode::CREATED, Json(todo))).
    todo!("TODO 5")
}

/// `GET /todos/{id}` : 200 et la tâche, 404 si elle n'existe pas.
async fn lire(
    State(etat): State<Arc<Etat>>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<Json<Todo>, ErreurApi> {
    // TODO 6 : let Path(id) = id?; puis iter().find(|t| t.id == id),
    // .cloned().map(Json).ok_or(ErreurApi::Introuvable(id)).
    todo!("TODO 6")
}

/// `PUT /todos/{id}` : modifie le titre et/ou l'état, 200 et la tâche à jour.
async fn modifier(
    State(etat): State<Arc<Etat>>,
    id: Result<Path<u64>, PathRejection>,
    corps: Result<Json<MajTodo>, JsonRejection>,
) -> Result<Json<Todo>, ErreurApi> {
    // TODO 7 : extrais id et corps avec ?. Si maj.titre est Some et vide après
    // trim -> TitreVide (avant de prendre le verrou). Puis write(),
    // iter_mut().find(...), ok_or(Introuvable(id))?, applique les champs
    // présents (if let Some(...)) et renvoie un clone.
    todo!("TODO 7")
}

/// `DELETE /todos/{id}` : 204 sans corps, ou 404.
async fn supprimer(
    State(etat): State<Arc<Etat>>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<StatusCode, ErreurApi> {
    // TODO 8 : write(), note la longueur, retain(|t| t.id != id) ; si la
    // longueur n'a pas changé -> Introuvable(id), sinon Ok(StatusCode::NO_CONTENT).
    todo!("TODO 8")
}

/// Sert l'API sur une écoute déjà ouverte jusqu'à ce que `arret` se termine.
///
/// # Errors
///
/// Renvoie l'erreur d'entrée/sortie de la boucle d'acceptation.
pub async fn servir<F>(ecoute: TcpListener, etat: Arc<Etat>, arret: F) -> std::io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    // TODO 9 : axum::serve(ecoute, routeur(etat)), puis
    // .with_graceful_shutdown(arret), puis .await.
    todo!("TODO 9")
}

/// Le client : `GET {base}/todos` et décodage de la liste.
///
/// # Errors
///
/// Erreur réseau, statut HTTP 4xx ou 5xx, ou JSON illisible.
pub async fn recuperer_todos(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<Todo>> {
    // TODO 10 : l'URL est format!("{}/todos", base.trim_end_matches('/')).
    // client.get(&url).send().await?, puis .error_for_status()? (un 404 doit
    // devenir une erreur), puis .json::<Vec<Todo>>().await?.
    todo!("TODO 10")
}

/// Le tableau affiché par `todo-api liste` : une ligne d'en-tête, puis une
/// ligne par tâche, colonnes de 4 et 6 caractères alignées à gauche :
///
/// ```text
/// ID  FAIT  TITRE
/// 1   [x]   sauvegarder pg-01
/// 12  [ ]   renouveler le certificat
/// ```
#[must_use]
pub fn format_tableau(todos: &[Todo]) -> String {
    // TODO 11 : format!("{:<4}{:<6}{}\n", ...) pour chaque ligne.
    todo!("TODO 11")
}
