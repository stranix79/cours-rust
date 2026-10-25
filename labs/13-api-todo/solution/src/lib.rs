// Labo 13, solution : une API REST de tâches avec axum, l'état partagé et
// le client qui l'interroge.
// Lancé par : (cd solution && cargo test), ou via src/main.rs :
//   (cd solution && cargo run -- serve --adresse 127.0.0.1:8080)
//
// La bibliothèque contient tout ce qui se teste : les types, l'état, le
// routeur, les handlers, le serveur avec arrêt propre, le client reqwest et
// l'affichage en tableau. main.rs ne fait que lire les arguments et brancher.

use std::future::Future; // le type du signal d'arrêt passé à servir()
use std::sync::atomic::{AtomicU64, Ordering}; // un compteur partagé sans verrou
use std::sync::{Arc, RwLock}; // l'état partagé entre les tâches tokio

use anyhow::Context; // .with_context() sur les erreurs du client
use axum::extract::rejection::{JsonRejection, PathRejection}; // les échecs d'extraction
use axum::extract::{Path, State}; // les extracteurs qu'on utilise
use axum::http::StatusCode; // 200, 201, 204, 400, 404
use axum::response::{IntoResponse, Response}; // ce qu'un handler peut renvoyer
use axum::routing::get; // get(), puis .post(), .put(), .delete() en chaîne
use axum::{Json, Router}; // le JSON en entrée comme en sortie, et le routeur
use serde::{Deserialize, Serialize}; // les dérivations serde (chapitre 12)
use serde_json::json; // json!({...}) pour les petits corps ad hoc
use tokio::net::TcpListener; // l'écoute TCP asynchrone

/// Une tâche, telle que l'API la renvoie et la reçoit en JSON.
///
/// Serialize et Deserialize : serde écrit et lit le JSON pour nous, les clés
/// sont les noms des champs (`id`, `titre`, `fait`). Clone parce qu'on renvoie
/// des copies de ce qui est sous le verrou, jamais une référence dessus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub titre: String,
    pub fait: bool,
}

/// Le corps de `POST /todos` : seulement le titre. Un client ne choisit ni
/// l'id ni l'état d'une tâche qu'il crée.
#[derive(Debug, Deserialize)]
pub struct NouveauTodo {
    pub titre: String,
}

/// Le corps de `PUT /todos/{id}` : chaque champ est facultatif. Option dit
/// « présent ou absent » mieux qu'une valeur sentinelle : `{"fait": true}`
/// coche la tâche sans toucher au titre.
#[derive(Debug, Deserialize)]
pub struct MajTodo {
    pub titre: Option<String>,
    pub fait: Option<bool>,
}

/// L'état partagé par toutes les requêtes.
///
/// axum sert chaque connexion dans une tâche tokio, sur plusieurs threads :
/// l'état doit être Send + Sync, d'où le RwLock autour du Vec (plusieurs
/// lecteurs en même temps, un seul écrivain) et l'AtomicU64 pour le compteur
/// (un `fetch_add` est atomique, pas besoin de verrou pour lui).
/// On prend le RwLock de la std et pas celui de tokio : on ne garde jamais le
/// verrou pendant un `.await`, et celui de la std est plus rapide.
#[derive(Debug)]
pub struct Etat {
    pub todos: RwLock<Vec<Todo>>,
    pub prochain_id: AtomicU64,
}

impl Etat {
    /// Un état vide, déjà dans son Arc : c'est sous cette forme que le
    /// routeur et les tests le partagent. Le premier id attribué sera 1.
    #[must_use]
    pub fn nouveau() -> Arc<Etat> {
        Arc::new(Etat {
            todos: RwLock::new(Vec::new()),
            prochain_id: AtomicU64::new(1),
        })
    }
}

/// Les erreurs de l'API. Un enum plutôt que des chaînes : le compilateur
/// vérifie que chaque cas a un statut HTTP, et les handlers renvoient
/// `Result<_, ErreurApi>` comme n'importe quelle fonction (chapitre 8).
#[derive(Debug)]
pub enum ErreurApi {
    TitreVide,
    Introuvable(u64),
    RequeteInvalide(String),
}

/// IntoResponse est le trait qui transforme une valeur en réponse HTTP.
/// L'implémenter pour notre erreur, c'est ce qui permet d'écrire `?` dans un
/// handler : axum appelle into_response() sur le Err.
impl IntoResponse for ErreurApi {
    fn into_response(self) -> Response {
        let (statut, message) = match self {
            ErreurApi::TitreVide => (
                StatusCode::BAD_REQUEST,
                "le titre est obligatoire".to_string(),
            ),
            ErreurApi::Introuvable(id) => (StatusCode::NOT_FOUND, format!("todo {id} introuvable")),
            ErreurApi::RequeteInvalide(message) => (StatusCode::BAD_REQUEST, message),
        };
        // Un tuple (StatusCode, Json) implémente déjà IntoResponse : statut,
        // Content-Type application/json, corps sérialisé.
        (statut, Json(json!({ "erreur": message }))).into_response()
    }
}

/// Quand le corps n'est pas du JSON valide, l'extracteur Json échoue avec
/// une JsonRejection (400, 415 ou 422 en texte brut). Cette conversion la
/// ramène à notre format d'erreur JSON, et `?` l'appelle tout seul.
impl From<JsonRejection> for ErreurApi {
    fn from(rejet: JsonRejection) -> Self {
        ErreurApi::RequeteInvalide(format!("JSON invalide : {}", rejet.body_text()))
    }
}

/// Même chose pour un `{id}` qui n'est pas un entier (`/todos/abc`).
impl From<PathRejection> for ErreurApi {
    fn from(rejet: PathRejection) -> Self {
        ErreurApi::RequeteInvalide(format!("id invalide : {}", rejet.body_text()))
    }
}

/// Construit le routeur de l'API. Il prend l'état en paramètre (au lieu de le
/// créer) pour que les tests puissent garder une main dessus.
///
/// Une route = un chemin + un MethodRouter : `get(f).post(g)` sur le même
/// chemin. Une méthode absente donne 405 tout seul, un chemin absent 404.
pub fn routeur(etat: Arc<Etat>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/todos", get(lister).post(creer))
        // axum 0.8 : les paramètres de chemin s'écrivent {id} (0.7 : :id).
        .route("/todos/{id}", get(lire).put(modifier).delete(supprimer))
        // with_state fournit l'Arc<Etat> à tous les handlers qui demandent
        // State<Arc<Etat>>. Le type est vérifié à la compilation : un handler
        // qui demande un autre type d'état ne compile pas.
        .with_state(etat)
}

/// `GET /healthz` : le service est vivant. Pas d'accès à l'état, donc pas
/// d'extracteur : un handler peut n'avoir aucun paramètre.
async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "statut": "ok" }))
}

/// `GET /todos` : la liste, dans l'ordre de création (les ids croissent).
async fn lister(State(etat): State<Arc<Etat>>) -> Json<Vec<Todo>> {
    // read() rend un garde ; .clone() copie le Vec, et le garde est relâché à
    // la fin de l'instruction : la sérialisation se fait sans verrou.
    let todos = etat.todos.read().expect("verrou empoisonné").clone();
    Json(todos)
}

/// `POST /todos` : 201 et la tâche créée, 400 si le JSON ou le titre ne va pas.
async fn creer(
    State(etat): State<Arc<Etat>>,
    // Result<Json<_>, JsonRejection> plutôt que Json<_> : on récupère
    // l'échec au lieu de laisser axum répondre en texte brut.
    corps: Result<Json<NouveauTodo>, JsonRejection>,
) -> Result<(StatusCode, Json<Todo>), ErreurApi> {
    let Json(nouveau) = corps?; // le ? passe par From<JsonRejection>
    let titre = nouveau.titre.trim();
    if titre.is_empty() {
        return Err(ErreurApi::TitreVide);
    }
    // fetch_add renvoie l'ancienne valeur et incrémente, atomiquement : deux
    // requêtes simultanées ne peuvent pas obtenir le même id. Relaxed suffit,
    // on ne synchronise rien d'autre que ce compteur.
    let id = etat.prochain_id.fetch_add(1, Ordering::Relaxed);
    let todo = Todo {
        id,
        titre: titre.to_string(),
        fait: false,
    };
    // Le garde d'écriture vit le temps de cette instruction seulement.
    etat.todos
        .write()
        .expect("verrou empoisonné")
        .push(todo.clone());
    Ok((StatusCode::CREATED, Json(todo)))
}

/// `GET /todos/{id}` : 200 et la tâche, 404 si elle n'existe pas.
async fn lire(
    State(etat): State<Arc<Etat>>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<Json<Todo>, ErreurApi> {
    let Path(id) = id?;
    let todos = etat.todos.read().expect("verrou empoisonné");
    // iter().find() rend une Option<&Todo> : cloned() en fait une
    // Option<Todo>, ok_or() une Result. Trois méthodes, aucun if.
    todos
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .map(Json)
        .ok_or(ErreurApi::Introuvable(id))
}

/// `PUT /todos/{id}` : modifie le titre et/ou l'état, 200 et la tâche à jour.
async fn modifier(
    State(etat): State<Arc<Etat>>,
    id: Result<Path<u64>, PathRejection>,
    corps: Result<Json<MajTodo>, JsonRejection>,
) -> Result<Json<Todo>, ErreurApi> {
    let Path(id) = id?;
    let Json(maj) = corps?;
    // On valide AVANT de prendre le verrou : rien à protéger pour ça.
    let titre = match maj.titre {
        Some(t) if t.trim().is_empty() => return Err(ErreurApi::TitreVide),
        Some(t) => Some(t.trim().to_string()),
        None => None,
    };
    let mut todos = etat.todos.write().expect("verrou empoisonné");
    // iter_mut : une référence mutable sur l'élément trouvé, qu'on modifie
    // en place dans le Vec. Le vérificateur d'emprunts sait que `todos` est
    // emprunté tant que `todo` vit.
    let todo = todos
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or(ErreurApi::Introuvable(id))?;
    if let Some(titre) = titre {
        todo.titre = titre;
    }
    if let Some(fait) = maj.fait {
        todo.fait = fait;
    }
    Ok(Json(todo.clone()))
}

/// `DELETE /todos/{id}` : 204 sans corps, ou 404.
async fn supprimer(
    State(etat): State<Arc<Etat>>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<StatusCode, ErreurApi> {
    let Path(id) = id?;
    let mut todos = etat.todos.write().expect("verrou empoisonné");
    let avant = todos.len();
    // retain garde ce qui ne correspond pas : si la longueur n'a pas bougé,
    // la tâche n'existait pas.
    todos.retain(|t| t.id != id);
    if todos.len() == avant {
        return Err(ErreurApi::Introuvable(id));
    }
    // Un StatusCode seul est une réponse valide : statut, pas de corps.
    Ok(StatusCode::NO_CONTENT)
}

/// Sert l'API sur une écoute déjà ouverte jusqu'à ce que `arret` se termine,
/// puis laisse finir les requêtes en cours et rend la main.
///
/// Recevoir le TcpListener (plutôt qu'une adresse) permet au test d'écouter
/// sur 127.0.0.1:0 et de lire le port choisi par le système. Recevoir le
/// signal d'arrêt sous forme de Future permet au test de l'envoyer lui-même,
/// sans Ctrl-C : main passe ctrl_c(), le test un canal oneshot.
///
/// # Errors
///
/// Renvoie l'erreur d'entrée/sortie de la boucle d'acceptation.
pub async fn servir<F>(ecoute: TcpListener, etat: Arc<Etat>, arret: F) -> std::io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::serve(ecoute, routeur(etat))
        .with_graceful_shutdown(arret)
        .await
}

/// Le client : `GET {base}/todos` et décodage de la liste.
///
/// Le Client est passé en paramètre : il contient un pool de connexions et
/// se crée une fois par programme, comme le http.Client de Go.
///
/// # Errors
///
/// Erreur réseau, statut HTTP 4xx ou 5xx, ou JSON qui n'est pas une liste de
/// tâches.
pub async fn recuperer_todos(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<Todo>> {
    let url = format!("{}/todos", base.trim_end_matches('/'));
    let todos = client
        .get(&url)
        .send()
        .await // la requête part ici : rien ne se passe avant le .await
        .with_context(|| format!("GET {url}"))?
        // Un 404 ou un 500 n'est pas une erreur pour reqwest :
        // error_for_status() le transforme en Err, comme raise_for_status().
        .error_for_status()?
        .json::<Vec<Todo>>()
        .await
        .context("réponse JSON illisible")?;
    Ok(todos)
}

/// Le tableau affiché par `todo-api liste`. Une fonction pure qui rend une
/// String : on la teste sans terminal, main n'a qu'à l'imprimer.
#[must_use]
pub fn format_tableau(todos: &[Todo]) -> String {
    let mut sortie = format!("{:<4}{:<6}{}\n", "ID", "FAIT", "TITRE");
    for t in todos {
        let case = if t.fait { "[x]" } else { "[ ]" };
        sortie.push_str(&format!("{:<4}{:<6}{}\n", t.id, case, t.titre));
    }
    sortie
}
