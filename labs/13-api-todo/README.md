# Labo 13 : API REST de tâches avec axum

**Chapitre** : [13. HTTP et async : tokio, reqwest, axum](../../cours/13-http-et-async.md)

## Objectif
Écrire une API REST complète (lister, créer, lire, modifier, supprimer des tâches) avec axum sur tokio : un routeur, des handlers async, les extracteurs `State`, `Path` et `Json`, un état partagé `Arc<Etat>` avec un `RwLock` et un compteur atomique, des erreurs qui deviennent des codes HTTP grâce à `IntoResponse`, un arrêt propre sur Ctrl-C et SIGTERM, et le client reqwest qui interroge l'API et affiche un tableau. C'est la transposition du labo 13 du cours Go : compare les deux quand tu as fini. Les tests t'apprennent la moitié du sujet : `tower::ServiceExt::oneshot` pour appeler le routeur sans ouvrir de port, et un vrai serveur sur `127.0.0.1:0` pour le bout en bout.

## Consignes
Tout se passe dans `src/lib.rs`, sauf le dernier TODO dans `src/main.rs`. Lance `cargo test` après chaque TODO et regarde ce qui passe au vert : au départ, tout échoue (`not yet implemented: TODO 1`), mais tout compile.

1. Lis `src/lib.rs` de haut en bas, puis `tests/api.rs`. Repère la fonction `appeler` des tests : elle construit une `Request`, la passe au routeur avec `oneshot`, lit le corps avec `collect()`. C'est l'équivalent de `httptest.NewRecorder` en Go.
2. **TODO 1**, `Etat::nouveau() -> Arc<Etat>` : un `Vec` vide sous `RwLock`, un `AtomicU64` à 1, le tout dans un `Arc`. Le test `healthz_repond_ok` passe déjà (`cargo test healthz`).
3. **TODO 2**, `IntoResponse for ErreurApi` : chaque variante donne un statut (400, 404, 400) et un corps `{"erreur": "..."}`. Sans ça, aucun `?` dans les handlers ne produit de réponse.
4. **TODO 3**, `routeur` : ajoute `/todos` (GET, POST) et `/todos/{id}` (GET, PUT, DELETE). axum répond tout seul 405 quand la méthode manque et 404 quand le chemin est inconnu : `cargo test methode`.
5. **TODO 4 et 5**, `lister` et `creer` : la lecture sous `read()`, l'écriture sous `write()`, l'id avec `fetch_add`. Le titre est nettoyé avec `trim()` ; vide, c'est un 400. Un corps qui n'est pas du JSON, ou où il manque `titre`, est un 400 en JSON grâce au `Result<Json<_>, JsonRejection>` et au `From` fourni. `cargo test creer`.
6. **TODO 6, 7 et 8**, `lire`, `modifier`, `supprimer` : `Path<u64>` pour l'id (un `abc` donne 400), `Introuvable(id)` pour 404. `modifier` n'applique que les champs présents (`Option`), et refuse un titre vide **avant** de prendre le verrou. `supprimer` répond 204 sans corps. `cargo test modifier`, `cargo test supprimer`.
7. Le test `creations_concurrentes_ids_uniques` lance cinquante créations dans cinquante tâches tokio sur quatre threads : si tes ids sont uniques et que rien ne se perd, ton état partagé est correct. Rust ne t'a de toute façon pas laissé écrire de course de données : essaie de remplacer le `RwLock` par un `Vec` nu dans `Etat`, et lis l'erreur du compilateur.
8. **TODO 9**, `servir` : `axum::serve(ecoute, routeur(etat)).with_graceful_shutdown(arret).await`. Une ligne, mais c'est elle que teste `bout_en_bout_sur_un_vrai_port` : le test envoie le signal d'arrêt par un canal oneshot et exige que le serveur rende la main en moins d'une seconde.
9. **TODO 10 et 11**, `recuperer_todos` (reqwest : `get`, `send`, `error_for_status`, `json`) et `format_tableau`. Puis lance le serveur dans un terminal et `cargo run -- liste` dans un autre.
10. **TODO 12**, dans `src/main.rs` : ajoute SIGTERM à Ctrl-C avec `tokio::select!`. Vérifie avec `kill <pid>` (sans `-9`) : le journal doit dire `arrêt propre`.
11. `cargo test` tout vert, puis retire la ligne `#![allow(...)]` en haut de `lib.rs`, et `cargo clippy --all-targets` puis `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                                   # tes tests (rouge tant que les TODO restent)
cargo test creer                             # seulement les tests dont le nom contient « creer »
cargo run -- serve --adresse 127.0.0.1:8080  # le serveur, Ctrl-C pour l'arrêter
cargo run -- liste --url http://127.0.0.1:8080
(cd solution && cargo test)                  # la solution (vert)
(cd solution && cargo run -- serve)
cargo clippy --all-targets && cargo fmt --check
```

Et pour jouer avec le serveur lancé, dans un autre terminal :
```bash
curl -s localhost:8080/todos
curl -s -X POST localhost:8080/todos -H 'content-type: application/json' -d '{"titre":"sauvegarder pg-01"}'
curl -s -X POST localhost:8080/todos -H 'content-type: application/json' -d '{"titre":"renouveler le certificat"}'
curl -s -i -X POST localhost:8080/todos -H 'content-type: application/json' -d '{"titre":""}'
curl -s -X PUT localhost:8080/todos/1 -H 'content-type: application/json' -d '{"fait":true}'
curl -s -i -X DELETE localhost:8080/todos/2
curl -s localhost:8080/todos/2
curl -s localhost:8080/todos/abc
curl -s -i -X PATCH localhost:8080/todos
```

## Sortie attendue
Côté client (les en-têtes `date` varient) :
```
$ curl -s localhost:8080/todos
[]
$ curl -s -X POST localhost:8080/todos -H 'content-type: application/json' -d '{"titre":"sauvegarder pg-01"}'
{"id":1,"titre":"sauvegarder pg-01","fait":false}
$ curl -s -X POST localhost:8080/todos -H 'content-type: application/json' -d '{"titre":"renouveler le certificat"}'
{"id":2,"titre":"renouveler le certificat","fait":false}
$ curl -s -i -X POST localhost:8080/todos -H 'content-type: application/json' -d '{"titre":""}'
HTTP/1.1 400 Bad Request
content-type: application/json
content-length: 37
date: Sun, 25 Oct 2026 17:24:08 GMT

{"erreur":"le titre est obligatoire"}
$ curl -s -X PUT localhost:8080/todos/1 -H 'content-type: application/json' -d '{"fait":true}'
{"id":1,"titre":"sauvegarder pg-01","fait":true}
$ curl -s -i -X DELETE localhost:8080/todos/2
HTTP/1.1 204 No Content
date: Sun, 25 Oct 2026 17:24:08 GMT

$ curl -s localhost:8080/todos/2
{"erreur":"todo 2 introuvable"}
$ curl -s localhost:8080/todos/abc
{"erreur":"id invalide : Invalid URL: Cannot parse `abc` to a `u64`"}
$ curl -s -i -X PATCH localhost:8080/todos
HTTP/1.1 405 Method Not Allowed
allow: GET,HEAD,POST
content-length: 0
date: Sun, 25 Oct 2026 17:24:08 GMT

$ cargo run -q -- liste
ID  FAIT  TITRE
1   [x]   sauvegarder pg-01
```

Côté serveur, puis Ctrl-C :
```
$ cargo run -q -- serve
écoute sur http://127.0.0.1:8080
^Csignal reçu, arrêt en cours (les requêtes en cours finissent)
arrêt propre
```

Et le client quand le serveur ne tourne plus : `anyhow` affiche la chaîne des causes, du contexte qu'on a ajouté jusqu'à l'erreur du système.
```
$ cargo run -q -- liste
Error: GET http://127.0.0.1:8080/todos

Caused by:
    0: error sending request for url (http://127.0.0.1:8080/todos)
    1: client error (Connect)
    2: tcp connect error
    3: Connection refused (os error 61)
```

## Pour aller plus loin
- Le 405 d'axum a un corps vide et le 404 d'une route inconnue aussi. Ajoute `.fallback(...)` au routeur pour que le 404 soit en JSON comme le reste, et regarde `tower_http::trace::TraceLayer` pour journaliser chaque requête avec son statut et sa durée (le middleware que tu écrivais à la main en Go).
- Ajoute `GET /todos?fait=true` avec l'extracteur `Query<Filtre>` (une struct `Filtre { fait: Option<bool> }` qui dérive `Deserialize`) et un test par valeur : `true`, `false`, absent, `oui` (400).
- Les tâches disparaissent à chaque redémarrage. Le chapitre 14 les met dans SQLite : remplace le `Vec` par une connexion rusqlite derrière un `Mutex`, sans toucher aux tests.
