# Labo 14 : un inventaire sur SQLite et PostgreSQL

**Chapitre** : [14. Bases de données : SQLite et PostgreSQL](../../cours/14-bases-de-donnees.md)

## Objectif
Écrire la couche d'accès aux données d'un petit inventaire : un trait `Depot` (ajouter, retirer, stock, lister), une implémentation SQLite avec rusqlite et une implémentation PostgreSQL avec la crate `postgres`, le même schéma embarqué dans le binaire avec `include_str!`, une transaction tout ou rien pour les retraits, et une CLI `inventaire-db` qui choisit le moteur selon le DSN. Les tests jouent le même scénario sur chaque moteur à travers le trait : SQLite en mémoire à chaque `cargo test`, PostgreSQL quand tu lui donnes un serveur. À la fin, tu as le patron de toutes tes couches de données en Rust : du SQL que tu écris toi-même, des paramètres liés, un `Option` pour l'absence, et le moteur caché derrière un trait.

Pourquoi la crate `postgres` et pas `tokio-postgres` : c'est la même bibliothèque (`postgres` enveloppe `tokio-postgres` et fait tourner son moteur async en interne), avec une API synchrone. Le dépôt a ainsi la même forme pour les deux moteurs, et le chapitre 14 montre à quoi ressemble la version async.

## Consignes
1. Lis `migrations/001.sql`, puis `src/lib.rs` et `tests/depot.rs`. La fonction `scenario(depot: &mut impl Depot)` est la spécification : elle est jouée telle quelle sur chaque moteur. Lance `cargo test` : ça compile, presque tout échoue (`not yet implemented: TODO 1`). Le test PostgreSQL passe déjà : sans `INVENTAIRE_DSN`, il ne fait rien.
2. **TODO 1**, `migrer` : `execute_batch(MIGRATION_001)`. Le schéma est dans le binaire, rejoué à chaque ouverture : `CREATE TABLE IF NOT EXISTS` le rend inoffensif.
3. **TODO 2**, `ajouter` : un `INSERT ... ON CONFLICT DO UPDATE` (un « upsert ») avec `params![nom, quantite]`. Une seule instruction, donc atomique sans transaction.
4. **TODO 3**, `retirer` : `self.conn.transaction()`, `query_row(...).optional()`, les deux erreurs (`article inconnu`, `stock insuffisant`), `UPDATE`, `commit`. Le test vérifie qu'après un retrait refusé, le stock n'a pas bougé : c'est le `Drop` de la transaction qui fait le `ROLLBACK`.
5. **TODO 4 et 5**, `stock` et `lister` : `.optional()` transforme « aucune ligne » en `None` ; `prepare` puis `query_map` puis `collect::<Result<Vec<_>, _>>()`. Écris aussi le `TryFrom<&rusqlite::Row>` qui projette une ligne vers un `Article`. `cargo test sqlite` doit passer, y compris le nom piégé `x'); DROP TABLE articles; --`.
6. **TODO 6**, `ouvrir(dsn)` : `sqlite::memory:`, `sqlite:chemin` ou `postgres://...`, rendu sous forme de `Box<dyn Depot>`. Le test vérifie que le message d'erreur d'un DSN inconnu ne contient pas le mot de passe. `cargo test ouvrir`.
7. Joue avec la CLI (bloc ci-dessous) puis regarde la base avec `sqlite3 inventaire.db 'SELECT * FROM articles'` si tu as le client SQLite. Le binaire, lui, n'a besoin de rien : SQLite est compilé dedans (`features = ["bundled"]`).
8. **TODO 7**, PostgreSQL : transpose les quatre méthodes. Il te faut un serveur. Si tu en as un, crée une base `inventaire` ; sinon, avec Docker ou Podman :
   ```bash
   docker run --rm -d --name pg-labo -p 5432:5432 -e POSTGRES_PASSWORD=labo -e POSTGRES_DB=inventaire postgres:17
   export INVENTAIRE_DSN=postgres://postgres:labo@127.0.0.1:5432/inventaire
   cargo test -- --nocapture
   ```
   Sans la variable, la sortie de `cargo test -- --nocapture` contient `PostgreSQL ignoré : INVENTAIRE_DSN absente`. Avec, le scénario tourne sur PostgreSQL et la ligne disparaît. Le mot de passe est dans une variable d'environnement, pas dans le code ni dans un fichier commité.
9. `cargo test` tout vert (avec et sans `INVENTAIRE_DSN`), retire la ligne `#![allow(...)]` de `lib.rs`, puis `cargo clippy --all-targets` et `cargo fmt --check`.

## Comment lancer
```bash
cargo test                                         # SQLite seulement
cargo test -- --nocapture                          # et le message « PostgreSQL ignoré »
INVENTAIRE_DSN=postgres://postgres:labo@127.0.0.1:5432/inventaire cargo test
cargo run -- ajouter vis 100                       # par défaut : sqlite:inventaire.db
cargo run -- --dsn sqlite::memory: lister          # une base jetable
INVENTAIRE_DSN=postgres://postgres:labo@127.0.0.1:5432/inventaire cargo run -- lister
(cd solution && cargo test)                        # la solution (vert)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run -q -- ajouter vis 100
vis : 100 en stock
$ cargo run -q -- ajouter ecrous 40
ecrous : 40 en stock
$ cargo run -q -- ajouter vis 20
vis : 120 en stock
$ cargo run -q -- retirer vis 30
vis : 90 en stock
$ cargo run -q -- retirer ecrous 41
Error: stock insuffisant pour ecrous : 40 en stock, 41 demandés
$ echo $?
1
$ cargo run -q -- stock boulons
boulons : article inconnu
$ echo $?
2
$ cargo run -q -- lister
ecrous                    40
vis                       90
$ cargo run -q -- ajouter vis 0
Error: quantité invalide : 0 (elle doit être positive)
$ cargo run -q -- --dsn postgres://postgres:labo@127.0.0.1:5999/inventaire lister
Error: connexion à PostgreSQL

Caused by:
    0: error connecting to server
    1: Connection refused (os error 61)
```

Le dernier message ne répète pas le DSN, et c'est voulu : il contient le mot de passe. Et `--help` affiche `[env: INVENTAIRE_DSN]` sans sa valeur, grâce à `hide_env_values`.

## Pour aller plus loin
- Ajoute une migration `002.sql` (une colonne `emplacement TEXT`) et une table `schema_version` : au démarrage, lis la version, applique les fichiers manquants dans une transaction chacun. C'est ce que font refinery et `sqlx migrate`, en soixante lignes.
- Lance deux `retirer` en même temps sur PostgreSQL depuis deux terminaux (ajoute un `std::thread::sleep` de deux secondes entre le SELECT et l'UPDATE) : avec `FOR UPDATE`, le second attend le premier. Retire `FOR UPDATE` et regarde le stock devenir faux.
- Le chapitre 16 garde des mesures dans SQLite à l'intérieur d'un serveur axum : une `Connection` n'est pas `Sync`, elle va dans un `Mutex`. Essaie de mettre un `DepotSqlite` dans l'état partagé du labo 13 sans `Mutex` et lis l'erreur du compilateur.
