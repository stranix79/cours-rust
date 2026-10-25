# 13. HTTP et async : tokio, reqwest, axum

*Rust de zéro à la prod : chapitre 13 sur 16.* ← [12. Fichiers, JSON, ligne de commande DevOps](12-fichiers-json-cli.md) · [Sommaire](../README.md) · [14. Bases de données : SQLite et PostgreSQL](14-bases-de-donnees.md) →

Au chapitre 11, tu as lancé des threads : un par tâche, ou un pool de quelques-uns. Ça marche très bien pour sonder cinquante ports. Ça marche moins bien pour tenir dix mille connexions HTTP ouvertes, dont neuf mille neuf cents attendent que le client lise sa réponse : dix mille threads, ce sont dix mille piles de 2 Mo et un ordonnanceur du noyau qui passe son temps à changer de contexte. C'est le problème que Go résout avec ses goroutines, Python avec asyncio, Node avec sa boucle d'événements. Rust le résout avec `async`/`.await`, et une particularité qui surprend tout le monde : le langage fournit la syntaxe, mais pas le moteur qui l'exécute.

Ce chapitre fait le tour dans l'ordre où tu en auras besoin : ce qu'est une future et pourquoi elle ne fait rien toute seule, tokio pour les exécuter, reqwest pour parler HTTP en client, axum pour écrire une API, l'arrêt propre, et les tests sans ouvrir de port. À la fin, tu as tout ce qu'il faut pour le [labo 13](../labs/13-api-todo/README.md), une API REST de tâches.

### 13.1 Une future est paresseuse

Une fonction `async fn` ne s'exécute pas quand on l'appelle. Elle rend une *future* : une valeur qui décrit un calcul à faire, qui avancera quand quelqu'un l'interrogera, et s'arrêtera à chaque `.await` en attendant qu'une donnée arrive. C'est `.await` qui lance la future et attend son résultat.

```rust
use std::time::Duration;

async fn sonder(nom: &str) -> u32 {
    println!("  sonde {nom} : début");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("  sonde {nom} : fin");
    42
}

#[tokio::main]
async fn main() {
    let future = sonder("web-01"); // rien ne s'affiche : la future est créée, pas lancée
    println!("future créée, pas encore attendue");
    let n = future.await; // c'est ici que sonder() commence
    println!("résultat : {n}");
}
```

```
$ cargo run
future créée, pas encore attendue
  sonde web-01 : début
  sonde web-01 : fin
résultat : 42
```

La ligne « future créée » sort avant « début ». En Go, `go sonder()` démarre la goroutine tout de suite ; en JavaScript, une `Promise` part dès sa création. En Rust, rien ne bouge tant que personne n'attend. Le compilateur le sait, et prévient quand on oublie :

```rust
async fn journaliser(message: &str) {
    println!("{message}");
}

#[tokio::main]
async fn main() {
    journaliser("démarrage");
    println!("fin");
}
```

```
$ cargo run
warning: unused implementer of `Future` that must be used
 --> src/bin/oubli.rs:7:5
  |
7 |     journaliser("démarrage");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: futures do nothing unless you `.await` or poll them

fin
```

« futures do nothing unless you `.await` or poll them » : tout est dit. Le programme compile, et `démarrage` n'est jamais affiché. Lis les avertissements, ils sont rarement là pour rien.

**Pourquoi c'est comme ça.** Une future Rust est une machine à états que le compilateur fabrique à partir de ta fonction : chaque `.await` est un état, les variables locales vivantes sont ses champs. Elle a une taille connue à la compilation, ne fait aucune allocation par elle-même, et ne coûte rien tant qu'on ne l'utilise pas. Pour avancer, elle a besoin d'un *exécuteur* qui l'interroge (`poll`) et d'un *réacteur* qui la réveille quand la socket qu'elle attend a des données. Ces deux pièces ne sont pas dans la bibliothèque standard, exprès : un serveur web, un microcontrôleur sans système d'exploitation et un navigateur (WebAssembly) n'ont pas besoin du même moteur. La std définit le trait `Future`, et chacun choisit son exécuteur. En pratique, pour un serveur ou un outil réseau, ce choix est fait : **tokio**. Les autres existent (smol, embassy pour l'embarqué), mais tout l'écosystème HTTP (hyper, axum, reqwest, tonic, sqlx) est construit sur tokio.

**Venant de Go :** le runtime Go est dans chaque binaire, invisible, et chaque fonction peut bloquer : l'ordonnanceur s'en arrange. En Rust, le runtime est une dépendance que tu ajoutes (`tokio = { version = "1", features = [...] }`), et la frontière entre code qui attend et code qui bloque est écrite dans les types. Plus de cérémonie, et en échange un binaire sans runtime quand tu n'en as pas besoin.

### 13.2 tokio : `#[tokio::main]`, `spawn`, `join!`

`#[tokio::main]` est une macro qui transforme ton `async fn main` en un `fn main` ordinaire : elle construit le moteur de tokio (un pool de threads de travail, un par cœur par défaut), lui donne ta future, et attend qu'elle finisse. C'est tout. Sans elle, une `async fn main` ne compile pas : `main` doit être une fonction normale, puisque personne ne ferait avancer la future.

Pour faire plusieurs choses à la fois, deux outils, qui ne font pas la même chose :

```rust
use std::time::{Duration, Instant};

async fn sonder(nom: &'static str, ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    format!("{nom} ok en {ms} ms")
}

#[tokio::main]
async fn main() {
    let debut = Instant::now();
    // join! : trois futures dans la MÊME tâche, attendues ensemble.
    let (a, b, c) = tokio::join!(sonder("web-01", 300), sonder("db-01", 200), sonder("dns", 100));
    println!("join!  : {a} | {b} | {c} ({} ms)", debut.elapsed().as_millis());

    let debut = Instant::now();
    // spawn : une tâche indépendante par cible, que l'exécuteur répartit sur
    // ses threads. Le JoinHandle se lit avec .await, comme un résultat.
    let mut taches = Vec::new();
    for (nom, ms) in [("web-01", 300), ("db-01", 200), ("dns", 100)] {
        taches.push(tokio::spawn(sonder(nom, ms)));
    }
    for t in taches {
        println!("spawn  : {}", t.await.expect("la tâche a paniqué"));
    }
    println!("spawn  : {} ms au total", debut.elapsed().as_millis());
}
```

```
$ cargo run
join!  : web-01 ok en 300 ms | db-01 ok en 200 ms | dns ok en 100 ms (301 ms)
spawn  : web-01 ok en 300 ms
spawn  : db-01 ok en 200 ms
spawn  : dns ok en 100 ms
spawn  : 301 ms au total
```

Les deux prennent 300 ms et pas 600 : les trois attentes se recouvrent. La différence est ailleurs.

- `join!` interroge ses futures à tour de rôle, dans la tâche courante. Pas d'allocation, pas de `'static` exigé, les futures peuvent emprunter des variables locales. Toutes tournent sur le même thread : c'est de la concurrence, pas du parallélisme.
- `tokio::spawn` crée une *tâche* : une future que l'exécuteur possède et peut déplacer d'un thread à l'autre. Elle doit donc être `Send` (chapitre 11) et `'static` (elle ne peut rien emprunter à la fonction qui l'a lancée, qui peut finir avant elle). Le `JoinHandle` rendu donne un `Result` : `Err` si la tâche a paniqué.

Une tâche tokio pèse quelques centaines d'octets, et tokio en fait tourner des centaines de milliers sur une poignée de threads. C'est ce qui fait qu'un serveur axum tient dix mille connexions : une tâche par connexion, comme Go a une goroutine par connexion.

**Venant de Go :** `tokio::spawn` est le `go` de Go, avec deux différences. Il rend un `JoinHandle` qu'on peut attendre (Go n'a rien d'équivalent, on passe par un channel ou un `WaitGroup`). Et la tâche ne peut pas capturer par référence une variable de la fonction appelante : `move` et `Arc` (chapitre 11), là où Go partageait en silence et laissait `-race` trouver les courses.

### 13.3 `select!`, `sleep`, `timeout`

Attendre la première de plusieurs choses est le pain quotidien d'un service : une réponse ou un délai, une requête ou un signal d'arrêt.

```rust
use std::time::Duration;

use tokio::time::{sleep, timeout};

async fn requete_lente() -> &'static str {
    sleep(Duration::from_secs(3)).await;
    "réponse"
}

#[tokio::main]
async fn main() {
    // timeout : la future, ou une erreur Elapsed si elle dépasse le délai.
    match timeout(Duration::from_millis(200), requete_lente()).await {
        Ok(r) => println!("timeout : {r}"),
        Err(e) => println!("timeout : {e}"),
    }

    // select! : la première des branches qui se termine gagne, les autres
    // sont abandonnées (leur future est détruite, donc annulée).
    tokio::select! {
        r = requete_lente() => println!("select! : {r}"),
        () = sleep(Duration::from_millis(200)) => println!("select! : trop lent, on abandonne"),
        _ = tokio::signal::ctrl_c() => println!("select! : Ctrl-C"),
    }
}
```

```
$ cargo run
timeout : deadline has elapsed
select! : trop lent, on abandonne
```

Le programme rend la main en 400 ms, pas en six secondes. Et c'est là que la paresse des futures paie : **annuler une future, c'est la détruire**. La `requete_lente` abandonnée par `select!` n'est plus interrogée, ses variables locales sont libérées par `Drop` (une connexion TCP en cours est fermée), et il n'y a rien d'autre à faire. Pas de `context.Context` à passer de fonction en fonction : celui qui attend décide, en arrêtant d'attendre.

**Piège :** l'annulation arrive à un `.await`, n'importe lequel. Une fonction qui écrit la moitié d'un fichier, fait un `.await`, puis écrit l'autre moitié, peut être coupée au milieu si quelqu'un la met dans un `select!`. Pour du code qui doit aller au bout (une écriture en base, un message envoyé), lance-le dans un `tokio::spawn` : une tâche n'est pas annulée quand on cesse d'attendre son `JoinHandle`.

**Venant de Go :** `select` sur des channels et `ctx.Done()` devient `tokio::select!` sur des futures. Le `context.WithTimeout` devient `tokio::time::timeout`. Et quand tu veux vraiment un jeton d'annulation à passer à plusieurs tâches (un signal, dix tâches à arrêter), c'est `tokio_util::sync::CancellationToken`, ou un canal `tokio::sync::watch` comme dans le projet final du chapitre 16.

### 13.4 Tâches contre threads : qui bloque qui

Une tâche tokio n'est interrompue qu'à ses `.await`. Entre deux `.await`, elle a le thread pour elle seule. Si elle appelle une fonction qui bloque (`std::thread::sleep`, une lecture de fichier avec `std::fs`, une requête SQL synchrone, un calcul de trois secondes), elle gèle son thread de travail et toutes les tâches qui attendaient d'y passer. Le voir sur un exécuteur à un seul thread :

```rust
use std::time::{Duration, Instant};

async fn battement(debut: Instant) {
    for _ in 0..3 {
        println!("  battement à {:>4} ms", debut.elapsed().as_millis());
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// Un seul thread d'exécution : on voit tout de suite qui bloque qui.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let debut = Instant::now();
    println!("avec std::thread::sleep dans une tâche :");
    let b = tokio::spawn(battement(debut));
    tokio::spawn(async {
        std::thread::sleep(Duration::from_millis(350)); // bloque LE thread
    });
    b.await.unwrap();

    let debut = Instant::now();
    println!("avec spawn_blocking :");
    let b = tokio::spawn(battement(debut));
    let lourd = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(350)); // sur un thread à part
        "calcul fini"
    });
    b.await.unwrap();
    println!("  {} à {} ms", lourd.await.unwrap(), debut.elapsed().as_millis());
}
```

```
$ cargo run
avec std::thread::sleep dans une tâche :
  battement à    0 ms
  battement à  355 ms
  battement à  456 ms
avec spawn_blocking :
  battement à    0 ms
  battement à  101 ms
  battement à  203 ms
  calcul fini à 355 ms
```

Le deuxième battement devait arriver à 100 ms ; il arrive à 355, parce que la tâche voisine a gardé le thread. Sur un moteur multi-thread, l'effet est dilué mais réel : quatre requêtes qui bloquent sur quatre threads, et ton serveur ne répond plus à personne. `spawn_blocking` envoie la fonction sur un pool de threads séparé, prévu pour attendre, et rend un `JoinHandle` qu'on attend avec `.await`. Le battement reprend son rythme.

La règle, simple :

- **De l'attente réseau, beaucoup de connexions** (serveur HTTP, client qui appelle cent API, proxy) : des tâches tokio. C'est pour ça que l'async existe.
- **Du calcul, ou une bibliothèque synchrone** (compression, hachage d'un gros fichier, rusqlite, un client PostgreSQL synchrone) : `spawn_blocking` depuis le code async, ou des threads ordinaires (chapitre 11) si le programme n'a pas besoin d'async du tout.
- **Un outil en ligne de commande qui lit trois fichiers et sort** : pas d'async. `std::fs` et des threads si besoin. Tirer tokio pour ça, c'est une minute de compilation pour rien.

Le projet final (chapitre 16) mélange les deux : le serveur axum tourne sur tokio, et le worker pool à threads du chapitre 11 est appelé tel quel dans un `spawn_blocking`.

**Venant du C :** tu as peut-être écrit une boucle `epoll_wait` (ou `select`, `kqueue`) avec des sockets non bloquantes et une machine à états par connexion. C'est exactement ce que fait tokio : son réacteur est un `epoll` (Linux) ou un `kqueue` (macOS), et chaque future est la machine à états que tu écrivais à la main, générée par le compilateur. Et la règle d'or est la même qu'en C : un appel bloquant dans la boucle d'événements gèle tout.

### 13.5 La coloration : async contamine

Une fonction async ne peut être attendue que depuis une fonction async :

```rust
async fn lire_config() -> String {
    tokio::fs::read_to_string("/etc/hosts").await.unwrap_or_default()
}

fn compter_lignes() -> usize {
    let texte = lire_config().await;
    texte.lines().count()
}
```

```
$ cargo build
error[E0728]: `await` is only allowed inside `async` functions and blocks
 --> src/bin/couleur.rs:6:31
  |
5 | fn compter_lignes() -> usize {
  | ---------------------------- this is not `async`
6 |     let texte = lire_config().await;
  |                               ^^^^^ only allowed inside `async` functions and blocks
```

On rend `compter_lignes` async, puis sa fonction appelante, puis la sienne, jusqu'à `main`. C'est ce qu'on appelle la *coloration* des fonctions : il y a les fonctions rouges (async) et les bleues (synchrones), une rouge ne s'appelle que depuis une rouge, et le rouge remonte la pile d'appels. Dans l'autre sens, la frontière se franchit une fois, en haut, avec un exécuteur :

```rust
async fn lire_config() -> String {
    tokio::fs::read_to_string("/etc/hosts").await.unwrap_or_default()
}

// Une fonction synchrone qui a besoin d'une fonction async : elle crée un
// exécuteur et attend dessus. C'est la frontière, à faire une fois, en haut.
fn compter_lignes() -> std::io::Result<usize> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let texte = rt.block_on(lire_config());
    Ok(texte.lines().count())
}

fn main() -> std::io::Result<()> {
    println!("/etc/hosts : {} lignes", compter_lignes()?);
    Ok(())
}
```

```
$ cargo run
/etc/hosts : 44 lignes
```

C'est ce que fait `#[tokio::main]`, et ce que fait `reqwest::blocking` en interne pour offrir un client synchrone. Mon avis : la coloration est un vrai coût, pas un détail. Elle coupe l'écosystème en deux (`std::fs` et `tokio::fs`, `reqwest` et `reqwest::blocking`, `postgres` et `tokio-postgres`), elle rend les traits async plus compliqués que les autres, et elle oblige à décider tôt. Mais elle a une contrepartie : en lisant une signature, tu sais si la fonction peut suspendre, donc où ton programme peut être coupé et où il peut attendre. Le bon usage s'en déduit : async **aux bords** (le serveur, le client réseau), synchrone **au milieu** (la logique métier, le parsing, les calculs), et une couche fine entre les deux. Une fonction qui calcule un taux de disponibilité n'a aucune raison d'être async.

**Venant de Go :** Go n'a pas de couleurs : toute fonction peut bloquer, et le runtime transforme le blocage en suspension de goroutine sans que tu le voies. C'est l'argument le plus solide de Go face à Rust pour les services réseau, et il est juste. Le prix côté Go : un runtime dans chaque binaire, et l'impossibilité de savoir, en lisant le code, où une fonction peut attendre.

**Venant de Python :** c'est exactement asyncio : `async def`, `await`, `asyncio.run(main())` qui est `block_on`, `asyncio.gather` qui est `join!`, `asyncio.create_task` qui est `tokio::spawn`, `asyncio.wait_for` qui est `timeout`, et la même règle « ne bloque pas la boucle ». Deux différences de taille : tokio fait tourner les tâches sur plusieurs cœurs pour de vrai (il n'y a pas de GIL), et le compilateur refuse les données partagées non protégées entre ces tâches.

### 13.6 Le client : reqwest

[reqwest](https://docs.rs/reqwest) est le client HTTP de l'écosystème : HTTP/1.1 et HTTP/2, TLS, JSON avec serde, délais, redirections, pool de connexions. Dans `Cargo.toml` :

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

`default-features = false` retire OpenSSL, que reqwest prend par défaut sur Linux : un binaire qui dépend d'OpenSSL demande `libssl-dev` à la compilation, `libssl` sur le serveur, et ne passe pas dans une image `scratch` (chapitre 15). `rustls-tls` fait le TLS en Rust pur, avec les certificats racines embarqués. `json` ajoute `.json()` dans les deux sens.

L'exemple démarre un petit serveur axum local (on verra comment en 13.7) pour avoir quelque chose à interroger, puis :

```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(2))
    .user_agent("sondes/1.0")
    .build()?;

let reponse = client.get(format!("{base}/hotes")).send().await?;
println!("statut : {}", reponse.status());
println!("type   : {:?}", reponse.headers()["content-type"]);
let hotes: Vec<Hote> = reponse.json().await?;
for h in &hotes {
    println!("  {} {}", h.nom, h.ip);
}

// Un 503 n'est pas une erreur pour reqwest : la réponse est valide.
let reponse = client.get(format!("{base}/panne")).send().await?;
println!("panne  : {} (is_success = {})", reponse.status(), reponse.status().is_success());
// error_for_status() en fait une Err, comme raise_for_status() en Python.
if let Err(e) = reponse.error_for_status() {
    println!("erreur : {e}");
}

// Le délai : une Err dont is_timeout() dit la nature.
match client.get(format!("{base}/lent")).send().await {
    Ok(r) => println!("lent   : {}", r.status()),
    Err(e) => println!("lent   : timeout ? {} ({e})", e.is_timeout()),
}
```

```
$ cargo run
statut : 200 OK
type   : "application/json"
  web-01 10.0.0.11
  db-01 10.0.0.21
panne  : 503 Service Unavailable (is_success = false)
erreur : HTTP status server error (503 Service Unavailable) for url (http://127.0.0.1:55252/panne)
lent   : timeout ? true (error sending request for url (http://127.0.0.1:55252/lent))
```

Quatre réflexes, les mêmes qu'en Go avec un vocabulaire différent :

- **Un `Client` par programme**, construit une fois et passé par référence (ou cloné : c'est un `Arc` à l'intérieur, le clone est gratuit). Il garde un pool de connexions ; en créer un par requête, c'est refaire la poignée de main TCP et TLS à chaque appel.
- **Toujours un `timeout`.** Le client par défaut n'en a pas, comme `http.DefaultClient` en Go. `reqwest::get(url)`, le raccourci sans client, non plus : réserve-le aux tests.
- **Le statut n'est pas une erreur.** `send().await?` échoue sur une panne réseau, un délai, un TLS refusé ; un 404 ou un 503 est une réponse. `error_for_status()?` quand un statut d'erreur doit devenir une `Err`.
- **Le corps se lit à part**, avec `.json::<T>()`, `.text()` ou `.bytes()`, chacun avec son `.await` : les en-têtes arrivent d'abord, le corps ensuite. Pas de `resp.Body.Close()` à oublier : la réponse est libérée par `Drop`, et la connexion retourne au pool.

Les en-têtes se posent par requête (`.header("Authorization", format!("Bearer {jeton}"))`, `.bearer_auth(jeton)`) ou pour tout le client (`.default_headers(...)` sur le builder). Un corps JSON part avec `.json(&valeur)` sur la requête, qui pose aussi `Content-Type`. Et `reqwest::Error` porte sa nature : `is_timeout()`, `is_connect()`, `is_status()`, `status()`. Pour un client d'infra qui réessaie, c'est ce qui décide : on réessaie sur un délai ou un 503, jamais sur un 400.

### 13.7 Le serveur : axum

[axum](https://docs.rs/axum) est le framework web de l'équipe tokio. Il ne fait pas grand-chose lui-même : il route les requêtes vers des fonctions async, et convertit leurs arguments et leurs résultats. Tout le reste (HTTP, connexions, TLS) est fait par hyper et tower, sur lesquels il est posé. Une API complète, un fichier, dont voici le cœur :

```rust
#[derive(Clone, Serialize, Deserialize)]
struct Hote {
    nom: String,
    ip: String,
    role: String,
}

#[derive(Deserialize)]
struct Filtre {
    role: Option<String>,
}

// GET /hotes?role=web : Query lit la chaîne de requête dans une struct.
async fn lister(State(etat): State<Etat>, Query(filtre): Query<Filtre>) -> Json<Vec<Hote>> {
    let hotes = etat.read().unwrap();
    let mut liste: Vec<Hote> = hotes
        .values()
        .filter(|h| filtre.role.as_ref().is_none_or(|r| &h.role == r))
        .cloned()
        .collect();
    liste.sort_by(|a, b| a.nom.cmp(&b.nom));
    Json(liste)
}

// GET /hotes/{nom} : Path extrait le segment, typé.
async fn lire(State(etat): State<Etat>, Path(nom): Path<String>) -> Result<Json<Hote>, ErreurApi> {
    let hotes = etat.read().unwrap();
    hotes.get(&nom).cloned().map(Json).ok_or(ErreurApi::Introuvable(nom))
}

// POST /hotes : Json désérialise le corps. Toujours le dernier extracteur :
// il consomme le corps de la requête.
async fn creer(State(etat): State<Etat>, Json(hote): Json<Hote>) -> Result<impl IntoResponse, ErreurApi> {
    if hote.nom.is_empty() {
        return Err(ErreurApi::Invalide("le nom est obligatoire"));
    }
    etat.write().unwrap().insert(hote.nom.clone(), hote.clone());
    Ok((StatusCode::CREATED, Json(hote)))
}

fn routeur(etat: Etat) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/hotes", get(lister).post(creer))
        .route("/hotes/{nom}", get(lire))
        .with_state(etat)
}
```

```
$ curl -s -i -X POST 127.0.0.1:8090/hotes -H 'content-type: application/json' -d '{"nom":"web-01","ip":"10.0.0.11","role":"web"}'
HTTP/1.1 201 Created
content-type: application/json
content-length: 46

{"nom":"web-01","ip":"10.0.0.11","role":"web"}
$ curl -s -X POST 127.0.0.1:8090/hotes -H 'content-type: application/json' -d '{"nom":"pg-01","ip":"10.0.0.21","role":"db"}'
{"nom":"pg-01","ip":"10.0.0.21","role":"db"}
$ curl -s '127.0.0.1:8090/hotes?role=db'
[{"nom":"pg-01","ip":"10.0.0.21","role":"db"}]
$ curl -s 127.0.0.1:8090/hotes/web-01
{"nom":"web-01","ip":"10.0.0.11","role":"web"}
$ curl -s -i 127.0.0.1:8090/hotes/nas
HTTP/1.1 404 Not Found
content-type: application/json
content-length: 34

{"erreur":"hôte nas introuvable"}
```

Ce qu'il faut voir, parce que c'est l'idée centrale d'axum : **un handler déclare ce dont il a besoin dans ses paramètres**, et axum le lui fournit. `State<T>` donne l'état partagé, `Path<T>` les segments de l'URL (`{nom}`, ou un tuple pour plusieurs), `Query<T>` la chaîne de requête, `Json<T>` le corps, `HeaderMap` les en-têtes. Chacun de ces *extracteurs* implémente un trait (`FromRequestParts`, ou `FromRequest` pour celui qui lit le corps, qui doit donc être le dernier). Et le type de retour est n'importe quoi qui implémente `IntoResponse` : une `String` (200, `text/plain`), un `Json<T>` (200, `application/json`), un `StatusCode` seul, un tuple `(StatusCode, Json<T>)`, un `Result<T, E>` dont les deux côtés implémentent `IntoResponse`. `impl IntoResponse` en retour, comme dans `creer`, veut dire « un type qui sait devenir une réponse, je ne l'écris pas ».

Et si l'extraction échoue, axum répond lui-même, avant d'appeler ton handler :

```
$ curl -s -i -X POST 127.0.0.1:8090/hotes -H 'content-type: application/json' -d '{"nom":"x"}'
HTTP/1.1 422 Unprocessable Entity
content-type: text/plain; charset=utf-8

Failed to deserialize the JSON body into the target type: missing field `ip` at line 1 column 11
$ curl -s -i -X POST 127.0.0.1:8090/hotes -d '{}'
HTTP/1.1 415 Unsupported Media Type
content-type: text/plain; charset=utf-8

Expected request with `Content-Type: application/json`
```

Pratique, mais en texte brut, alors que le reste de l'API parle JSON. Pour garder la main, on demande l'extraction sous forme de `Result` : `corps: Result<Json<T>, JsonRejection>` dans les paramètres, puis `let Json(t) = corps?;` avec un `From<JsonRejection>` vers ton type d'erreur. C'est ce que fait le labo 13.

**Venant de Go :** un `http.HandlerFunc` reçoit `(w, r)` et fait tout lui-même : lire `r.PathValue("id")`, `strconv.Atoi`, `json.NewDecoder(r.Body).Decode`, `w.WriteHeader`, encoder. Le handler axum reçoit des valeurs déjà converties et rend une valeur ; l'écriture de la réponse est faite après lui, à partir du type. Un handler qui oublie d'écrire son statut, ou l'écrit après le corps, ne peut pas exister. Et les routes se déclarent comme avec le `ServeMux` de Go 1.22 : `{nom}` dans le chemin, une méthode par appel (`get(f).post(g)`), et 405 avec l'en-tête `Allow` quand la méthode n'existe pas.

**Venant de Python :** si tu as écrit du FastAPI, tu es chez toi. `Path`, `Query`, le corps typé par un modèle pydantic qui devient une struct serde, `Depends` qui devient `State`, les 422 sur un corps invalide. Ce qui manque : la doc OpenAPI générée (des crates comme utoipa la fournissent). Ce qu'on gagne : la vérification se fait à la compilation, et le serveur tient cent fois plus de requêtes par cœur.

### 13.8 L'état partagé et les erreurs

Le type `Etat` de l'exemple est un alias : `type Etat = Arc<RwLock<HashMap<String, Hote>>>;`. `Arc` parce que chaque requête tourne dans sa propre tâche, peut-être sur un autre thread, et que toutes doivent voir la même table. `RwLock` parce que plusieurs lectures simultanées sont sans danger, et qu'une écriture doit être seule. C'est exactement le chapitre 11 ; `with_state(etat)` le donne au routeur, et chaque handler qui demande `State<Etat>` en reçoit un clone (un clone d'`Arc`, donc un compteur incrémenté, pas une copie de la table).

Le `RwLock` de la std, pas celui de tokio : tant qu'on ne garde pas le verrou pendant un `.await`, celui de la std est plus rapide et suffit. Et si on le garde, le compilateur proteste. Le handler fautif :

```rust
async fn incrementer(State(compteur): State<Arc<Mutex<u64>>>) -> String {
    let mut n = compteur.lock().unwrap();
    *n += 1;
    tokio::time::sleep(Duration::from_millis(10)).await; // le garde vit encore ici
    format!("{n}\n")
}
```

```
$ cargo build
error[E0277]: the trait bound `fn(State<Arc<Mutex<u64>>>) -> ... {incrementer}: Handler<_, _>` is not satisfied
   --> src/bin/verrou_await.rs:18:33
    |
 18 |         .route("/compter", post(incrementer))
    |                            ---- ^^^^^^^^^^^ unsatisfied trait bound
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
```

Le message est obscur : axum dit que ta fonction n'est pas un handler, sans dire pourquoi. Suis son conseil : ajoute `#[axum::debug_handler]` au-dessus de la fonction (la feature `macros` d'axum), et la vraie raison apparaît :

```
$ cargo build
error: future cannot be sent between threads safely
  --> src/bin/verrou_await.rs:8:1
   |
 8 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ future returned by `incrementer` is not `Send`
   |
   = help: within `impl Future<Output = String>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, u64>`
note: future is not `Send` as this value is used across an await
  --> src/bin/verrou_await.rs:12:51
   |
10 |     let mut n = compteur.lock().unwrap();
   |         ----- has type `std::sync::MutexGuard<'_, u64>` which is not `Send`
11 |     *n += 1;
12 |     tokio::time::sleep(Duration::from_millis(10)).await; // le garde vit encore ici
   |                                                   ^^^^^ await occurs here, with `mut n` maybe used later
```

Le garde du `Mutex` vit à travers un `.await`, il fait donc partie de la future ; or un `MutexGuard` de la std n'est pas `Send` (il doit être rendu par le thread qui l'a pris), donc la future non plus, et tokio refuse de la déplacer entre ses threads. Le compilateur vient de t'éviter un interblocage : une tâche suspendue avec le verrou en poche, et toutes les autres qui l'attendent. La correction : relâcher le verrou avant d'attendre, en le confinant à un bloc.

```rust
async fn incrementer(State(compteur): State<Arc<Mutex<u64>>>) -> String {
    let n = {
        let mut n = compteur.lock().unwrap();
        *n += 1;
        *n
    }; // le garde est détruit ici, avant le .await
    tokio::time::sleep(Duration::from_millis(10)).await;
    format!("{n}\n")
}
```

```
$ for i in 1 2 3; do curl -s -X POST 127.0.0.1:8092/compter; done
1
2
3
```

Quand il faut vraiment tenir un verrou pendant une attente (une connexion partagée qu'on utilise en async), c'est `tokio::sync::Mutex`, dont le garde est `Send` et dont `.lock().await` suspend la tâche au lieu de bloquer le thread. C'est plus lent, et rarement nécessaire.

**Les erreurs** suivent le chapitre 8 : un `enum` pour les cas, et `IntoResponse` pour dire à quel statut chaque cas correspond.

```rust
enum ErreurApi {
    Introuvable(String),
    Invalide(&'static str),
}

impl IntoResponse for ErreurApi {
    fn into_response(self) -> Response {
        let (statut, message) = match self {
            ErreurApi::Introuvable(nom) => (StatusCode::NOT_FOUND, format!("hôte {nom} introuvable")),
            ErreurApi::Invalide(m) => (StatusCode::BAD_REQUEST, m.to_string()),
        };
        (statut, Json(json!({ "erreur": message }))).into_response()
    }
}
```

Avec ça, un handler rend `Result<Json<Hote>, ErreurApi>`, utilise `?` et `ok_or` comme n'importe quelle fonction, et le format d'erreur est le même partout. Ajoute un `impl From<anyhow::Error> for ErreurApi` qui rend un 500 (en journalisant le détail, sans le montrer au client), et `?` marche aussi sur les erreurs de ta base ou de tes fichiers. Et une route `/healthz` qui répond `ok` sans toucher à rien : c'est elle que Docker, Kubernetes ou ton load balancer interrogent pour savoir si le processus est vivant.

### 13.9 Arrêt propre

`docker stop` envoie SIGTERM, attend dix secondes, puis SIGKILL. Kubernetes fait pareil avec trente secondes. Entre les deux, un service bien élevé cesse d'accepter des connexions, termine les requêtes en cours, et sort. axum le fait en une ligne :

```rust
axum::serve(ecoute, routeur(etat))
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("Ctrl-C");
        println!("arrêt demandé, les requêtes en cours finissent");
    })
    .await?;
println!("arrêt propre");
```

```
$ cargo run
écoute sur http://127.0.0.1:8090
^Carrêt demandé, les requêtes en cours finissent
arrêt propre
```

`with_graceful_shutdown` prend une future : quand elle se termine, le serveur ferme son écoute, attend que les connexions en cours aient fini leur requête, et `serve(...).await` rend la main. Une future, pas un signal : dans les tests, on lui passe la réception d'un canal `oneshot`, et le test arrête le serveur quand il veut, sans Ctrl-C. `ctrl_c()` n'attend que SIGINT ; pour SIGTERM, `tokio::signal::unix::signal(SignalKind::terminate())` et un `select!` entre les deux, avec un `#[cfg(unix)]` pour que Windows compile : la version complète est dans `src/main.rs` du labo 13.

**Venant de Go :** c'est `signal.NotifyContext` plus `srv.Shutdown(ctx)`, sans la goroutine pour `ListenAndServe`, sans `http.ErrServerClosed` à filtrer, sans contexte neuf à créer pour le délai. Si tu veux borner l'attente des requêtes en cours, enveloppe le tout dans un `tokio::time::timeout`.

### 13.10 Tester une API sans ouvrir de port

Un `Router` axum est un *service* au sens de tower : une fonction async qui prend une requête et rend une réponse. On peut donc l'appeler directement, sans réseau, avec `tower::ServiceExt::oneshot` (dans `[dev-dependencies]` : `tower = { version = "0.5", features = ["util"] }` et `http-body-util = "0.1"` pour lire le corps).

```rust
#[tokio::test]
async fn creer_puis_lire_sans_port() {
    let etat = Etat::default();
    let corps = r#"{"nom":"pg-01","ip":"10.0.0.21","role":"db"}"#;
    let requete = Request::post("/hotes")
        .header("content-type", "application/json")
        .body(Body::from(corps))
        .unwrap();
    let reponse = routeur(etat.clone()).oneshot(requete).await.unwrap();
    assert_eq!(reponse.status(), StatusCode::CREATED);

    let requete = Request::get("/hotes/pg-01").body(Body::empty()).unwrap();
    let reponse = routeur(etat).oneshot(requete).await.unwrap();
    assert_eq!(reponse.status(), StatusCode::OK);
    let octets = reponse.into_body().collect().await.unwrap().to_bytes();
    let hote: Hote = serde_json::from_slice(&octets).unwrap();
    assert_eq!(hote.ip, "10.0.0.21");
}

#[tokio::test]
async fn sur_un_vrai_port() {
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", ecoute.local_addr().unwrap());
    tokio::spawn(axum::serve(ecoute, routeur(Etat::default())).into_future());
    let reponse = reqwest::get(format!("{base}/hotes/inconnu")).await.unwrap();
    assert_eq!(reponse.status(), reqwest::StatusCode::NOT_FOUND);
    println!("{base} -> {}", reponse.text().await.unwrap());
}
```

```
$ cargo test -- --nocapture
running 2 tests
test tests::creer_puis_lire_sans_port ... ok
http://127.0.0.1:55734 -> {"erreur":"hôte inconnu introuvable"}
test tests::sur_un_vrai_port ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`#[tokio::test]` est le `#[tokio::main]` des tests : chaque test a son propre exécuteur. Deux familles, comme `httptest.NewRecorder` et `httptest.NewServer` en Go :

- **`oneshot`** pour les handlers : pas de port, pas de réseau, des centaines de tests par seconde. Le routeur est consommé par `oneshot`, on le reconstruit autour du même `Etat` pour la requête suivante : c'est l'état qui porte les données.
- **Un vrai port** `127.0.0.1:0` (le système choisit un port libre, `local_addr()` dit lequel) pour tester un client, ou ce qui exige une vraie connexion : l'arrêt propre, les délais, le comportement de reqwest. Le serveur tourne dans une tâche `tokio::spawn`.

Ajoute `flavor = "multi_thread"` (`#[tokio::test(flavor = "multi_thread", worker_threads = 4)]`) quand tu veux que les tâches tournent vraiment en parallèle : le labo 13 l'utilise pour lancer cinquante créations simultanées et vérifier qu'aucun id n'est attribué deux fois.

### 13.11 Autour : tower-http et tracing

axum ne journalise rien et n'ajoute aucun en-tête de lui-même. Les *middlewares* viennent de tower et de [tower-http](https://docs.rs/tower-http), qui s'empilent sur le routeur avec `.layer(...)`. Les deux qu'on met presque toujours :

```rust
use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // RUST_LOG décide de ce qui s'affiche : RUST_LOG=debug, RUST_LOG=tower_http=debug...
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(false)
        .init();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .layer(TraceLayer::new_for_http()) // une ligne par requête : méthode, chemin, statut, durée
        .layer(CorsLayer::permissive()); // CORS ouvert : pour du développement seulement
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:8091").await?;
    tracing::info!(adresse = %ecoute.local_addr()?, "écoute");
    axum::serve(ecoute, app).await?;
    Ok(())
}
```

```
$ RUST_LOG=info,tower_http=debug cargo run
2026-10-25T18:22:39.109252Z  INFO trace: écoute adresse=127.0.0.1:8091
2026-10-25T18:22:39.657532Z DEBUG request{method=GET uri=/healthz version=HTTP/1.1}: tower_http::trace::on_request: started processing request
2026-10-25T18:22:39.657620Z DEBUG request{method=GET uri=/healthz version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=0 ms status=200
2026-10-25T18:22:39.670734Z DEBUG request{method=GET uri=/rien version=HTTP/1.1}: tower_http::trace::on_request: started processing request
2026-10-25T18:22:39.670790Z DEBUG request{method=GET uri=/rien version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=0 ms status=404
```

`TraceLayer` est le middleware de journalisation que tu écrivais à la main en Go, avec le statut et la durée. `CorsLayer` gère les requêtes `OPTIONS` d'un navigateur ; `permissive()` autorise tout, à remplacer en production par la liste de tes origines. tower-http a aussi la compression, les délais par requête, les limites de taille de corps, le service de fichiers statiques.

[tracing](https://docs.rs/tracing) est le journal structuré de l'écosystème tokio : `tracing::info!(cible = %nom, duree_ms, "sonde terminée")`, des champs plutôt que du texte, et des *spans* qui donnent un contexte à tout ce qui se passe dedans (le `request{method=GET uri=/healthz}` de la sortie). `tracing-subscriber` décide où et comment ça s'écrit, en texte ou en JSON, filtré par la variable `RUST_LOG`. Le chapitre 15 y revient pour la production.

### 13.12 Pour le labo

Le [labo 13](../labs/13-api-todo/README.md) assemble tout ce chapitre : une API REST de tâches (`GET/POST /todos`, `GET/PUT/DELETE /todos/{id}`, `/healthz`) avec un état `Arc<Etat>` sous `RwLock` et un compteur `AtomicU64`, des erreurs JSON par `IntoResponse`, un arrêt propre sur Ctrl-C et SIGTERM, un client reqwest qui affiche un tableau, et une suite de tests qui couvre chaque route et chaque code d'erreur avec `oneshot`, plus un test de bout en bout sur un vrai port. C'est la transposition du labo 13 du cours Go : compare les deux une fois fini.

### À retenir

- Une `async fn` rend une future qui ne fait rien tant qu'on ne l'attend pas avec `.await` ; annuler une future, c'est la détruire.
- La std définit `Future` mais pas d'exécuteur ; pour le réseau, l'exécuteur est tokio, et `#[tokio::main]` le met en place.
- `join!` attend plusieurs futures dans la même tâche ; `tokio::spawn` crée une tâche indépendante, `Send + 'static`, qui rend un `JoinHandle`.
- `select!` prend la première branche finie et annule les autres ; `timeout` borne une attente ; `sleep` de tokio, jamais `std::thread::sleep` dans du code async.
- Ce qui bloque (calcul, fichiers, bibliothèque synchrone) va dans `spawn_blocking` ; l'async se met aux bords du programme, la logique reste synchrone.
- reqwest : un `Client` réutilisé, toujours un `timeout`, `error_for_status()` pour les 4xx et 5xx, `rustls-tls` sans OpenSSL.
- axum : un handler déclare ses extracteurs (`State`, `Path`, `Query`, `Json` en dernier) et rend un `impl IntoResponse` ; un `enum` d'erreurs avec `IntoResponse` donne un format d'erreur unique.
- L'état partagé est un `Arc<RwLock<…>>` de la std, jamais gardé pendant un `.await` ; `#[axum::debug_handler]` traduit les erreurs obscures.
- `with_graceful_shutdown` pour l'arrêt propre, `oneshot` pour tester sans port, `127.0.0.1:0` pour tester avec un vrai serveur, `TraceLayer` et `tracing` pour journaliser.

---

← [12. Fichiers, JSON, ligne de commande DevOps](12-fichiers-json-cli.md) · [Sommaire](../README.md) · [14. Bases de données : SQLite et PostgreSQL](14-bases-de-donnees.md) →
