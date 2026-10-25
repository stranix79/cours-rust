# 14. Bases de données : SQLite et PostgreSQL

*Rust de zéro à la prod : chapitre 14 sur 16.* ← [13. HTTP et async : tokio, reqwest, axum](13-http-et-async.md) · [Sommaire](../README.md) · [15. Qualité, build, release, Docker](15-qualite-build-release.md) →

L'API du chapitre 13 garde ses tâches en mémoire : on redémarre, tout disparaît. Il est temps de parler à une vraie base. Pour toi qui administres PostgreSQL, Rust a une qualité rare : il ne te cache rien. Pas d'ORM imposé, pas de couche magique. Tu écris ton SQL, tu passes tes paramètres, tu lis tes lignes, et le système de types fait deux choses que tu apprécieras : il t'oblige à traiter le NULL, et il refuse de lire un `BIGINT` dans un entier trop petit.

Ce chapitre suit l'ordre dans lequel tu vas te cogner aux choses : SQLite avec rusqlite (requêtes, NULL, transactions, injection, migrations), PostgreSQL avec la crate `postgres` (connexion, types, erreurs du serveur), puis la version async, sqlx et ses requêtes vérifiées à la compilation, les pools, et un trait pour cacher le moteur et tester sans base. Tout sert au [labo 14](../labs/14-sqlite-postgres/README.md).

### 14.1 Pas de `database/sql` : une crate par moteur

Go a une API générique dans la bibliothèque standard, `database/sql`, et des pilotes qui s'y branchent. Rust n'en a pas. Chaque moteur a sa crate, avec son API, écrite au plus près de ce que le moteur sait faire :

| Moteur | Crate synchrone | Crate async |
|---|---|---|
| SQLite | `rusqlite` | (rusqlite dans `spawn_blocking`, ou sqlx) |
| PostgreSQL | `postgres` | `tokio-postgres` |
| MySQL | `mysql` | `mysql_async` |
| plusieurs | | `sqlx` (PostgreSQL, SQLite, MySQL) |

Le prix : passer de SQLite à PostgreSQL, c'est changer de crate, pas seulement de DSN. Le gain : chaque crate parle son moteur sans traduction. rusqlite expose les fonctions SQLite définies en Rust, les sauvegardes à chaud, les blobs incrémentaux ; `postgres` expose `COPY`, les tableaux, `LISTEN`/`NOTIFY`, les types `INET` et `JSONB`. Et quand ton code doit vraiment tourner sur deux moteurs, tu le dis toi-même avec un trait (section 14.11), ce qui reste plus clair qu'une API générique qui fait semblant que tous les SQL se ressemblent.

Mon conseil pour choisir : rusqlite pour un outil ou un service qui embarque sa base, `postgres` pour un programme synchrone qui parle à PostgreSQL (un outil d'administration, un job), `tokio-postgres` ou sqlx dans un service axum.

**Venant de Go :** `database/sql` plus un pilote (`modernc.org/sqlite`, `pgx/stdlib`) devient une crate par moteur, sans enregistrement de pilote ni `import _`. Le pool que `*sql.DB` te donnait d'office devient une crate à part (14.10), ou est fourni par sqlx. Et le mode natif de pgx, celui que tu prenais pour les vrais types PostgreSQL, c'est ce que `postgres` et `tokio-postgres` font tout le temps.

### 14.2 rusqlite : ouvrir, écrire, lire

SQLite, tu connais : une base complète dans un fichier, sans serveur. Dans `Cargo.toml` :

```toml
rusqlite = { version = "0.41", features = ["bundled"] }
```

`bundled` compile le code C de SQLite et le lie dans ton binaire. Sans cette feature, rusqlite cherche la `libsqlite3` du système à l'édition de liens, dans la version qu'elle a, et ton binaire en dépend sur le serveur. Avec, rien à installer nulle part, et la même version de SQLite partout. La compilation prend une minute de plus la première fois, une seule fois.

```rust
use rusqlite::{Connection, OptionalExtension, params};

fn main() -> rusqlite::Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE hotes (nom TEXT PRIMARY KEY, ip TEXT NOT NULL, os TEXT);
         CREATE INDEX hotes_ip ON hotes (ip);",
    )?;

    // execute : une instruction, des paramètres liés, le nombre de lignes touchées.
    let n = conn.execute(
        "INSERT INTO hotes (nom, ip, os) VALUES (?1, ?2, ?3)",
        params!["web-01", "10.0.0.11", "debian"],
    )?;
    println!("insérée : {n} ligne, rowid {}", conn.last_insert_rowid());
    conn.execute("INSERT INTO hotes (nom, ip, os) VALUES (?1, ?2, ?3)", params!["db-01", "10.0.0.21", "debian"])?;
    conn.execute("INSERT INTO hotes (nom, ip, os) VALUES (?1, ?2, NULL)", params!["nas", "10.0.0.5"])?;

    // query_row : une ligne, lue par une fermeture.
    let ip: String = conn.query_row("SELECT ip FROM hotes WHERE nom = ?1", params!["db-01"], |l| l.get(0))?;
    println!("db-01   : {ip}");

    // Pas de ligne : une erreur QueryReturnedNoRows, que .optional() change en None.
    let absent: Option<String> = conn
        .query_row("SELECT ip FROM hotes WHERE nom = ?1", params!["pg-99"], |l| l.get(0))
        .optional()?;
    println!("pg-99   : {absent:?}");

    // prepare + query_map : N lignes, un itérateur de Result.
    let mut requete = conn.prepare("SELECT nom, os FROM hotes ORDER BY nom")?;
    let lignes = requete.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, Option<String>>(1)?)))?;
    for ligne in lignes {
        let (nom, os) = ligne?;
        println!("  {nom:<8} {}", os.as_deref().unwrap_or("(inconnu)"));
    }

    // Lire un NULL dans un String : une erreur de type, pas une chaîne vide.
    let erreur = conn.query_row("SELECT os FROM hotes WHERE nom = 'nas'", [], |l| l.get::<_, String>(0));
    println!("NULL dans un String : {erreur:?}");
    Ok(())
}
```

```
$ cargo run
insérée : 1 ligne, rowid 1
db-01   : 10.0.0.21
pg-99   : None
  db-01    debian
  nas      (inconnu)
  web-01   debian
NULL dans un String : Err(InvalidColumnType(0, "os", Null))
```

Les quatre appels à connaître :

- `execute_batch(sql)` : plusieurs instructions séparées par des points-virgules, sans paramètres. Pour le DDL et les migrations.
- `execute(sql, params)` : une instruction, rend le nombre de lignes touchées. `last_insert_rowid()` donne l'id de la dernière insertion.
- `query_row(sql, params, |ligne| ...)` : exactement une ligne. La fermeture lit les colonnes avec `ligne.get(index)` ou `ligne.get("nom")`, et le type de destination décide de la conversion. Pas de ligne, c'est `Err(QueryReturnedNoRows)` ; `.optional()` (le trait `OptionalExtension`) en fait un `Ok(None)`, ce qui est presque toujours ce qu'on veut.
- `prepare(sql)` puis `query_map(params, |ligne| ...)` : N lignes, sous forme d'itérateur de `Result`. On le parcourt avec `for`, ou on le collecte avec `collect::<Result<Vec<_>, _>>()?` (chapitre 8), qui s'arrête à la première erreur.

**Piège :** une `Connection` rusqlite est `Send` mais pas `Sync` : on peut la déplacer vers un autre thread, pas la partager entre deux. Dans un serveur axum, elle va dans un `Mutex` (l'état partagé du chapitre 13), ou chaque tâche bloquante ouvre la sienne. Le compilateur te le rappellera si tu oublies : c'est le genre d'erreur que Go laisse passer et que `-race` trouve en production.

### 14.3 Le NULL, c'est `Option`

La dernière ligne de la sortie est la plus importante du chapitre : lire un NULL dans un `String` est une **erreur**, `InvalidColumnType(0, "os", Null)`. Pas une chaîne vide, pas une valeur par défaut silencieuse. Une colonne qui peut être NULL se lit dans un `Option<T>` : `None` pour NULL, `Some(valeur)` sinon. Et dans l'autre sens, passer un `None` en paramètre envoie un NULL.

C'est la même idée que tout le cours depuis le chapitre 6 : l'absence est dans le type, et le compilateur t'empêche de l'oublier. Le schéma dit `os TEXT` (nullable), ta struct dit `os: Option<String>`, et le code qui affiche doit décider quoi faire du `None` (ici, `unwrap_or("(inconnu)")`). Si tu veux qu'une colonne ne puisse jamais être NULL, c'est `NOT NULL` dans le schéma et `String` dans la struct, et les deux disent la même chose.

**Venant de Go :** `sql.NullString`, `sql.NullInt64`, ou un `*string` deviennent `Option<String>`, `Option<i64>`. Même sens, sans le champ `Valid` qu'on oublie de tester.

**Venant de Python :** `sqlite3` et psycopg rendent `None` pour NULL, et rien ne t'oblige à le traiter : le `None` se promène jusqu'au `.upper()` qui lève `AttributeError` trois fonctions plus loin. Ici, le type `Option` t'arrête à la lecture.

### 14.4 Transactions : le `Drop` fait le ROLLBACK

`conn.transaction()` ouvre une transaction et rend un objet `Transaction` qui emprunte la connexion de façon exclusive. Tout passe par lui. Il a deux façons de finir : `commit()`, qui le consomme, ou être détruit sans commit, et son `Drop` fait le `ROLLBACK`.

```rust
use anyhow::{Context, bail};
use rusqlite::{Connection, params};

fn virer(conn: &mut Connection, de: &str, vers: &str, go: i64) -> anyhow::Result<()> {
    let tx = conn.transaction()?; // BEGIN
    tx.execute("UPDATE quotas SET go = go - ?1 WHERE projet = ?2", params![go, de])?;
    let reste: i64 = tx.query_row("SELECT go FROM quotas WHERE projet = ?1", params![de], |l| l.get(0))?;
    if reste < 0 {
        bail!("quota insuffisant pour {de} : il resterait {reste} Go");
    } // ici, tx est détruite sans commit : ROLLBACK
    tx.execute("UPDATE quotas SET go = go + ?1 WHERE projet = ?2", params![go, vers])?;
    tx.commit().context("commit")?; // COMMIT, et tx est consommée
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE quotas (projet TEXT PRIMARY KEY, go INTEGER NOT NULL);
         INSERT INTO quotas VALUES ('sondes', 100), ('logs', 50);",
    )?;
    println!("virer 30 : {:?}", virer(&mut conn, "sondes", "logs", 30));
    println!("virer 90 : {:?}", virer(&mut conn, "sondes", "logs", 90).map_err(|e| e.to_string()));
    let mut requete = conn.prepare("SELECT projet, go FROM quotas ORDER BY projet")?;
    for ligne in requete.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, i64>(1)?)))? {
        let (projet, go) = ligne?;
        println!("  {projet:<8} {go} Go");
    }
    Ok(())
}
```

```
$ cargo run
virer 30 : Ok(())
virer 90 : Err("quota insuffisant pour sondes : il resterait -20 Go")
  logs     80 Go
  sondes   70 Go
```

Le second virement a bien débité `sondes` de 90 avant de constater le solde négatif. Le `bail!` est sorti de la fonction, `tx` a été détruite, le `ROLLBACK` est parti, et la table est comme après le premier virement. Même chose pour chaque `?` entre le `BEGIN` et le `COMMIT` : n'importe quelle erreur annule tout, sans une ligne de plus.

Et le vérificateur d'emprunts ajoute une garantie que Go n'a pas : tant que `tx` vit, `conn` est empruntée en exclusivité. Un `conn.execute(...)` au milieu de la transaction, qui partirait hors transaction, ne compile pas. Après `tx.commit()`, `tx` n'existe plus : un `tx.execute(...)` après le commit ne compile pas non plus.

**Venant de Go :** le patron `tx, err := db.BeginTx(...)`, `defer tx.Rollback()`, `return tx.Commit()` est fait par le compilateur. Le `defer` est le `Drop`, et l'erreur « j'ai utilisé `db` au lieu de `tx` dans la transaction » n'existe plus.

**Venant de Python :** c'est le `with conn:` de `sqlite3`, qui commit à la sortie normale et rollback sur exception. Ici, la sortie normale doit être explicite (`commit()`), ce qui évite le commit d'une transaction qu'on croyait avoir annulée.

### 14.5 Jamais de `format!` dans du SQL

```rust
use rusqlite::{Connection, params};

fn main() -> rusqlite::Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE comptes (login TEXT);
         INSERT INTO comptes VALUES ('admin'), ('gilles'), ('sauvegarde');",
    )?;
    let saisie = "x' OR '1'='1"; // ce que tape un visiteur malveillant

    let requete = format!("SELECT count(*) FROM comptes WHERE login = '{saisie}'");
    let n: i64 = conn.query_row(&requete, [], |l| l.get(0))?;
    println!("{requete}\n  lignes : {n}");

    let n: i64 = conn.query_row("SELECT count(*) FROM comptes WHERE login = ?1", params![saisie], |l| l.get(0))?;
    println!("SELECT count(*) FROM comptes WHERE login = ?1\n  lignes : {n}");
    Ok(())
}
```

```
$ cargo run
SELECT count(*) FROM comptes WHERE login = 'x' OR '1'='1'
  lignes : 3
SELECT count(*) FROM comptes WHERE login = ?1
  lignes : 0
```

Avec `format!`, la saisie est devenue du SQL et la condition est toujours vraie : trois comptes au lieu de zéro. Avec un paramètre lié, la valeur part à côté de la requête, jamais interprétée. Rust ne t'empêche pas d'écrire la première version (une `String` est une `String`), mais clippy et toute revue de code le devraient. Les valeurs passent **toujours** par les paramètres. La seule chose qu'on ne peut pas lier, c'est un nom de table ou de colonne : là, on valide contre une liste blanche avant de concaténer.

La syntaxe dépend du moteur. SQLite accepte `?`, `?1`, `:nom`, `$1` ; PostgreSQL veut `$1`, `$2`. Le labo écrit `?1` pour SQLite et `$1` pour PostgreSQL, avec le même SQL par ailleurs. Côté Rust, `params![a, b]` (rusqlite) et `&[&a, &b]` (postgres) font la même chose : une liste de références à des valeurs qui savent se convertir en type SQL (les traits `ToSql`).

**Venant de Python :** c'est le `cursor.execute("... WHERE login = ?", (saisie,))` de `sqlite3` et le `%s` de psycopg, et le même piège : un f-string dans `execute` est une injection, quel que soit le langage.

### 14.6 Le schéma dans le binaire

`CREATE TABLE IF NOT EXISTS` suffit le premier jour. Pour qu'un binaire porte son schéma sans fichier à copier à côté, on l'embarque à la compilation avec `include_str!`, qui lit un fichier relatif au source et en fait un `&'static str` :

```rust
const MIGRATION_001: &str = include_str!("../migrations/001.sql");

fn migrer(conn: rusqlite::Connection) -> anyhow::Result<Self> {
    conn.execute_batch(MIGRATION_001).context("migration 001")?;
    Ok(DepotSqlite { conn })
}
```

C'est l'extrait du labo 14, où `migrations/001.sql` crée la table avec `IF NOT EXISTS` : rejouer la migration à chaque démarrage ne casse rien. Le jour où il faut une colonne de plus sur une table qui a des données, il faut de vraies migrations numérotées, appliquées une fois chacune, avec une table qui note où on en est. La crate [refinery](https://docs.rs/refinery) le fait pour rusqlite et postgres, `sqlx migrate` pour sqlx, et le labo propose de l'écrire toi-même en piste bonus : soixante lignes, et tu sauras ce que font les autres.

**Piège :** `Connection::open_in_memory()` donne une base propre à **cette** connexion. Une deuxième connexion ouverte avec `open_in_memory()` voit une autre base, vide. C'est parfait pour les tests (une base neuve par test, en quelques millisecondes), et c'est un piège dès qu'on met un pool devant : chaque connexion du pool aurait sa propre base.

### 14.7 PostgreSQL avec la crate `postgres`

La crate [postgres](https://docs.rs/postgres) est un client synchrone : `Client::connect` ouvre une connexion, et chaque appel attend la réponse du serveur. Sous le capot, c'est `tokio-postgres` (14.8) qui fait le travail, avec un petit moteur tokio caché ; l'API est la même aux `.await` près. Pour un outil d'administration ou un job qui fait ses requêtes l'une après l'autre, c'est le plus simple.

Les exemples tournent contre un PostgreSQL 16 local sur le port 5498 (un `initdb` jetable), DSN dans la variable `PG_DSN` :

```rust
use postgres::{Client, NoTls};

fn main() -> anyhow::Result<()> {
    // Le DSN vient de l'environnement, jamais du code.
    let dsn = std::env::var("PG_DSN").unwrap_or_else(|_| "postgres://postgres@127.0.0.1:5432/postgres".into());
    let mut client = Client::connect(&dsn, NoTls)?;

    let version: String = client.query_one("SHOW server_version", &[])?.get(0);
    println!("serveur : {version}");

    client.batch_execute(
        "DROP TABLE IF EXISTS hotes;
         CREATE TABLE hotes (nom TEXT PRIMARY KEY, ip INET NOT NULL, cpu INTEGER, vu TIMESTAMPTZ);",
    )?;
    // $1, $2 : les paramètres de PostgreSQL. Les valeurs sont des &(dyn ToSql + Sync).
    let n = client.execute(
        "INSERT INTO hotes (nom, ip, cpu) VALUES ($1, $2::TEXT::INET, $3), ($4, $5::TEXT::INET, NULL)",
        &[&"web-01", &"10.0.0.11", &4_i32, &"nas", &"10.0.0.5"],
    )?;
    println!("insérées : {n}");

    for ligne in client.query("SELECT nom, host(ip), cpu FROM hotes ORDER BY nom", &[])? {
        let nom: &str = ligne.get(0);
        let ip: String = ligne.get(1);
        let cpu: Option<i32> = ligne.get(2); // NULL devient None
        println!("  {nom:<8} {ip:<10} cpu={cpu:?}");
    }

    // Un INTEGER lu dans un i64 : erreur de type explicite avec try_get.
    let ligne = client.query_one("SELECT cpu FROM hotes WHERE nom = 'web-01'", &[])?;
    if let Err(e) = ligne.try_get::<_, i64>(0) {
        println!("try_get i64 : {:#}", anyhow::Error::from(e));
    }

    // Une erreur du serveur porte son code SQLSTATE.
    let err = client
        .execute("INSERT INTO hotes (nom, ip) VALUES ('web-01', '10.0.0.12')", &[])
        .expect_err("doublon");
    if let Some(db) = err.as_db_error() {
        println!("code {} : {}", db.code().code(), db.message());
        println!("doublon ? {}", *db.code() == postgres::error::SqlState::UNIQUE_VIOLATION);
    }
    Ok(())
}
```

```
$ PG_DSN=postgres://postgres@127.0.0.1:5498/inventaire cargo run
serveur : 16.15 (Homebrew)
insérées : 2
  nas      10.0.0.5   cpu=None
  web-01   10.0.0.11  cpu=Some(4)
try_get i64 : error deserializing column 0: cannot convert between the Rust type `i64` and the Postgres type `int4`
code 23505 : la valeur d'une clé dupliquée rompt la contrainte unique « hotes_pkey »
doublon ? true
```

Ce qu'il faut retenir de cette sortie, dans l'ordre :

- `query` rend un `Vec<Row>`, `query_one` exactement une ligne (sinon erreur), `query_opt` une `Option<Row>`, `execute` le nombre de lignes touchées, `batch_execute` plusieurs instructions sans paramètres.
- **Les types sont vérifiés, dans les deux sens, et strictement.** Un `INTEGER` (`int4`) se lit dans un `i32`, un `BIGINT` (`int8`) dans un `i64`, un `TEXT` dans un `String` ou un `&str`, un `BOOLEAN` dans un `bool`, un `BYTEA` dans un `Vec<u8>`. Lire un `int4` dans un `i64` est une erreur, alors que la valeur tiendrait : la crate refuse de deviner. `get` panique sur une erreur de type, `try_get` la rend. Dans un service, `try_get`.
- **Dans l'autre sens aussi.** Sans les `::TEXT::INET`, l'insertion échoue avec ``cannot convert between the Rust type `&str` and the Postgres type `inet` ``. PostgreSQL annonce le type de chaque paramètre, et un `&str` n'est pas un `INET`. On caste côté SQL, ou on active une feature qui fait correspondre un type Rust (`with-chrono-0_4` pour `TIMESTAMPTZ`, `with-serde_json-1` pour `JSONB`, `with-uuid-1` pour `UUID`).
- **Les erreurs du serveur portent leur SQLSTATE.** `err.as_db_error()` donne le `DbError` avec `code()`, `message()`, `constraint()`. Le message est en français parce que le serveur de test a la locale du Mac : ne compare jamais un message, compare le code. `SqlState::UNIQUE_VIOLATION` est le `23505` que tu connais.

`NoTls` veut dire connexion en clair : bien pour `127.0.0.1` ou un réseau Docker interne, pas ailleurs. Pour TLS, il faut une crate en plus (`postgres-native-tls`, ou `tokio-postgres-rustls` avec rustls comme pour reqwest), passée à la place de `NoTls`. Le DSN accepte la forme URL ou la forme `host=... user=... dbname=...` de libpq, mais pas les variables `PGHOST` et `PGPASSWORD` ni `~/.pgpass` : si tu y tiens, lis-les toi-même.

**Venant du C :** sous rusqlite, ce sont les mêmes appels que tu faisais avec `sqlite3.h` : `sqlite3_open`, `sqlite3_prepare_v2`, `sqlite3_bind_*`, `sqlite3_step`, `sqlite3_column_*`, `sqlite3_finalize`. rusqlite les enveloppe, et le `finalize` que tu oubliais est fait par `Drop`. La crate `postgres`, elle, ne passe pas par libpq : elle parle le protocole réseau de PostgreSQL en Rust pur. Pas de `libpq.so` à installer, et une compilation croisée qui marche.

### 14.8 Projeter des lignes vers des structs

Lire chaque colonne à la main dans chaque requête devient vite répétitif. Le trait standard de la conversion qui peut échouer, `TryFrom`, fait très bien l'affaire : on l'implémente une fois par struct, et la requête devient une ligne. Extrait du labo 14 :

```rust
impl TryFrom<&postgres::Row> for Article {
    type Error = postgres::Error;

    fn try_from(ligne: &postgres::Row) -> Result<Self, Self::Error> {
        Ok(Article {
            nom: ligne.try_get("nom")?,
            quantite: ligne.try_get("quantite")?,
        })
    }
}

fn lister(&self) -> anyhow::Result<Vec<Article>> {
    let lignes = self
        .client
        .borrow_mut()
        .query("SELECT nom, quantite FROM articles ORDER BY nom", &[])?;
    let articles = lignes
        .iter()
        .map(Article::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(articles)
}
```

Les colonnes sont lues par nom, pas par position : un `SELECT` réordonné ne casse rien. Le même `impl TryFrom<&rusqlite::Row<'_>>` sert côté SQLite. Ce n'est pas un ORM, c'est la conversion écrite une fois, testée, et rangée à côté du type.

(Le `borrow_mut()` vient de ce que le `Client` de `postgres` demande `&mut self` même pour lire, alors que le trait du labo promet `&self` pour les lectures : le client est rangé dans un `RefCell`, chapitre 10. Le labo l'explique.)

### 14.9 La version async : tokio-postgres

Dans un service axum, une requête SQL synchrone gèle un thread de travail de tokio (chapitre 13). Deux réponses : la mettre dans `spawn_blocking`, ou prendre le client async. `tokio-postgres` a la même API que `postgres`, avec des `.await`, et une particularité :

```rust
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dsn = std::env::var("PG_DSN")?;
    // connect rend le client ET la connexion : la connexion est une future
    // qui fait le travail réseau, il faut la lancer dans sa propre tâche.
    let (client, connexion) = tokio_postgres::connect(&dsn, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connexion.await {
            eprintln!("connexion PostgreSQL perdue : {e}");
        }
    });
    let lignes = client.query("SELECT nom FROM hotes ORDER BY nom", &[]).await?;
    let noms: Vec<&str> = lignes.iter().map(|l| l.get(0)).collect();
    println!("hôtes : {noms:?}");
    Ok(())
}
```

```
$ PG_DSN=postgres://postgres@127.0.0.1:5498/inventaire cargo run
hôtes : ["nas", "web-01"]
```

`connect` rend deux objets : le `Client`, qui envoie les requêtes, et la `Connection`, une future qui lit et écrit sur la socket. Tant que personne ne l'attend, rien ne part (13.1) : on la lance dans une tâche à part, et le client lui parle par un canal interne. Un avantage inattendu : plusieurs tâches peuvent utiliser le même `Client` en même temps (il est `Sync`), et leurs requêtes sont mises en file sur la connexion, en *pipeline*.

### 14.10 sqlx : le SQL vérifié à la compilation

[sqlx](https://docs.rs/sqlx) prend le problème à l'envers. Tu écris ton SQL dans une macro, et **pendant la compilation**, la macro se connecte à ta base de développement, prépare la requête, et vérifie les colonnes, leurs types et leur nullabilité contre ta struct.

```rust
use sqlx::postgres::PgPoolOptions;

struct Hote {
    nom: String,
    cpu: Option<i32>, // la colonne accepte NULL : sqlx exige un Option
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Un pool : jusqu'à 5 connexions, partagées par toutes les tâches.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    // query_as! se connecte à la base PENDANT LA COMPILATION, prépare la
    // requête, et vérifie colonnes, types et nullabilité contre la struct.
    let hotes = sqlx::query_as!(Hote, "SELECT nom, cpu FROM hotes WHERE nom <> $1 ORDER BY nom", "rien")
        .fetch_all(&pool)
        .await?;
    for h in hotes {
        println!("{:<8} cpu={:?}", h.nom, h.cpu);
    }
    Ok(())
}
```

```
$ DATABASE_URL=postgres://postgres@127.0.0.1:5498/inventaire cargo run
nas      cpu=None
web-01   cpu=Some(4)
```

Une faute de frappe dans une colonne, et c'est la compilation qui échoue, avec le message du serveur :

```
$ cargo build
error: error returned from database: la colonne « cpus » n'existe pas
  --> src/main.rs:17:17
   |
17 |     let hotes = sqlx::query_as!(Hote, "SELECT nom, cpus FROM hotes WHERE nom <> $1 ORDER BY nom", "rien")
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

Et un `cpu: i32` dans la struct, alors que la colonne accepte NULL :

```
$ cargo build
error[E0277]: the trait bound `i32: From<Option<i32>>` is not satisfied
  --> src/main.rs:17:17
   |
17 |     let hotes = sqlx::query_as!(Hote, "SELECT nom, cpu FROM hotes WHERE nom <> $1 ORDER BY nom", "rien")
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<Option<i32>>` is not implemented for `i32`
```

C'est puissant, et ça a un coût. Sans base joignable à la compilation, rien ne compile :

```
$ cargo build
error: set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache
```

La réponse s'appelle le **mode hors ligne** : `cargo sqlx prepare` (de l'outil `sqlx-cli`) enregistre la description de chaque requête dans un dossier `.sqlx/` que tu commites, et avec `SQLX_OFFLINE=true` la CI et le `docker build` compilent sans base. Il faut relancer `prepare` à chaque requête modifiée, et la CI doit vérifier que le cache est à jour (`cargo sqlx prepare --check`). Les fonctions sans `!` (`sqlx::query_as::<_, Hote>(...)`) ne vérifient rien à la compilation et ne demandent rien : on peut mélanger.

Deux remarques de terrain. sqlx compile lourd (une centaine de crates de plus), et ses versions récentes suivent Rust de près : au moment où j'écris, sqlx 0.9 exige Rust 1.94, et ce chapitre utilise donc sqlx 0.8 avec Rust 1.93 (`cargo` le dit clairement : `sqlx-postgres@0.9.0 requires rustc 1.94.0`). Et sqlx fournit le pool, les migrations (`sqlx::migrate!`) et trois moteurs derrière une API commune : c'est ce qui se rapproche le plus de `database/sql`.

Quand le choisir : un service async qui parle à PostgreSQL, avec beaucoup de requêtes, et une équipe qui accepte la base de développement et le cache `.sqlx/`. Pour un outil synchrone, ou un service qui a dix requêtes, rusqlite ou `postgres` avec un `TryFrom` par struct restent plus simples.

**Venant de Go :** c'est l'idée de sqlc (le SQL vérifié contre le schéma, et du code typé qui en sort), mais faite par le compilateur au lieu d'un générateur : pas de `sqlc generate` à lancer, et pas de fichier généré à commiter, sauf le cache hors ligne.

### 14.11 Les pools : deadpool et r2d2

Une connexion PostgreSQL est un processus côté serveur. Dans un service, on n'en ouvre pas une par requête HTTP (une poignée de main, une authentification, un fork côté serveur, à chaque fois), et on n'en partage pas une seule entre toutes les tâches (elles attendraient en file). On garde un **pool** : quelques connexions ouvertes, prêtées à la demande, rendues après usage.

- **sqlx** a le sien (`PgPoolOptions`, ci-dessus).
- **deadpool** (`deadpool-postgres`) est le pool async pour `tokio-postgres` : `pool.get().await?` rend un objet qui se comporte comme un `Client` et retourne au pool à son `Drop`.
- **r2d2** est le pool synchrone historique, avec des adaptateurs pour rusqlite (`r2d2_sqlite`) et `postgres` (`r2d2_postgres`) : `pool.get()?` bloque jusqu'à ce qu'une connexion soit libre.

Dans les trois cas, la connexion rendue par `get` revient au pool toute seule quand elle sort de la portée : le `Drop` encore, et aucune connexion oubliée qui fuit.

**Venant de Go :** le pool de `*sql.DB` n'a pas de maximum par défaut ; ceux de Rust en ont un (10 connexions pour sqlx, par exemple), mais lis la documentation de chacun plutôt que de deviner, et fixe-le toi-même. Dans les deux cas, ta règle de DBA s'applique : la taille du pool multipliée par le nombre d'instances du service doit rester sous `max_connections`, avec de la marge pour toi et pour les sauvegardes.

### 14.12 Un trait pour cacher le moteur, et tester sans base

Le reste du programme n'a pas à savoir s'il parle à SQLite ou à PostgreSQL. Il a besoin de quatre opérations. Un trait (chapitre 7) les nomme, et chaque moteur l'implémente :

```rust
use std::collections::HashMap;

use anyhow::bail;

/// Ce que le programme attend d'un stockage, quel que soit le moteur.
trait Depot {
    fn ajouter(&mut self, nom: &str, quantite: i64) -> anyhow::Result<()>;
    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>>;
}

/// Une implémentation en mémoire : pour les tests de ce qui est AU-DESSUS du
/// dépôt (une API, une règle métier), sans base du tout.
#[derive(Default)]
struct DepotMemoire {
    articles: HashMap<String, i64>,
}

impl Depot for DepotMemoire {
    fn ajouter(&mut self, nom: &str, quantite: i64) -> anyhow::Result<()> {
        if quantite <= 0 {
            bail!("quantité invalide : {quantite}");
        }
        *self.articles.entry(nom.to_string()).or_insert(0) += quantite;
        Ok(())
    }
    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>> {
        Ok(self.articles.get(nom).copied())
    }
}

/// La logique métier ne connaît que le trait.
fn reapprovisionner(depot: &mut impl Depot, nom: &str, seuil: i64) -> anyhow::Result<bool> {
    let actuel = depot.stock(nom)?.unwrap_or(0);
    if actuel >= seuil {
        return Ok(false);
    }
    depot.ajouter(nom, seuil - actuel)?;
    Ok(true)
}

fn main() -> anyhow::Result<()> {
    let mut depot = DepotMemoire::default();
    depot.ajouter("vis", 30)?;
    println!("vis    : commandé ? {}", reapprovisionner(&mut depot, "vis", 100)?);
    println!("vis    : {:?}", depot.stock("vis")?);
    println!("vis    : commandé ? {}", reapprovisionner(&mut depot, "vis", 100)?);
    println!("ecrous : commandé ? {}", reapprovisionner(&mut depot, "ecrous", 50)?);
    println!("ecrous : {:?}", depot.stock("ecrous")?);
    Ok(())
}
```

```
$ cargo run
vis    : commandé ? true
vis    : Some(100)
vis    : commandé ? false
ecrous : commandé ? true
ecrous : Some(50)
```

Deux façons de s'en servir, et le labo 14 fait les deux. **`impl Depot`** en paramètre (ou un générique `<D: Depot>`) : le compilateur fabrique une version de la fonction par moteur, sans coût à l'exécution ; c'est ce que fait `scenario(depot: &mut impl Depot)` dans les tests, joué sur SQLite en mémoire puis sur PostgreSQL. **`Box<dyn Depot>`** quand le moteur n'est connu qu'à l'exécution : la fonction `ouvrir(dsn)` du labo lit le préfixe du DSN et rend l'un ou l'autre, et `main` ne voit que le trait.

Pour les tests, SQLite en mémoire est souvent un meilleur double que la `HashMap` : c'est du vrai SQL, avec les vraies contraintes (`CHECK`, `UNIQUE`), en quelques millisecondes. La limite est honnête : SQLite n'est pas PostgreSQL. Il n'a pas `FOR UPDATE`, pas de tableaux, un typage laxiste (une chaîne entre dans une colonne `INTEGER`), et la collation de son `ORDER BY` n'est pas celle de ton serveur. Dès que ton SQL utilise une spécificité de PostgreSQL, teste sur PostgreSQL : le labo le fait quand la variable `INVENTAIRE_DSN` est définie, et passe en le disant quand elle ne l'est pas.

### 14.13 Où mettre le DSN

Un DSN PostgreSQL contient un mot de passe. Il ne va ni dans le code, ni dans un fichier commité, ni dans un message d'erreur. Il vient d'une variable d'environnement, lue au démarrage :

```rust
/// Où est la base : sqlite:fichier.db, sqlite::memory: ou postgres://...
#[arg(
    long,
    env = "INVENTAIRE_DSN",
    hide_env_values = true,
    default_value = "sqlite:inventaire.db"
)]
dsn: String,
```

Avec clap (chapitre 12), `env = "INVENTAIRE_DSN"` lit la variable quand l'option manque, et `hide_env_values` empêche `--help` d'afficher sa valeur. Côté erreurs, le labo met `.context("connexion à PostgreSQL")` et pas le DSN : la chaîne d'erreur finit dans un journal, et le journal dans un outil que d'autres lisent. Un test du labo vérifie justement qu'un DSN refusé ne réapparaît pas dans le message. En conteneur, la variable vient de `docker run -e`, d'un fichier `.env` non commité, ou d'un secret de l'orchestrateur ; le chapitre 15 y revient.

### 14.14 Pour le labo

Le [labo 14](../labs/14-sqlite-postgres/README.md) te fait écrire un inventaire derrière un trait `Depot` (ajouter, retirer, stock, lister) : une implémentation rusqlite avec le schéma embarqué par `include_str!`, un retrait en transaction tout ou rien, des erreurs claires pour un stock insuffisant ou un article inconnu ; une implémentation PostgreSQL avec la crate `postgres`, les mêmes garanties et un `FOR UPDATE` ; et une CLI `inventaire-db` qui choisit le moteur selon `--dsn` ou `INVENTAIRE_DSN`. Les tests jouent le même scénario sur les deux moteurs, PostgreSQL seulement si tu lui en donnes un.

### À retenir

- Rust n'a pas de `database/sql` : une crate par moteur (rusqlite, `postgres`, `tokio-postgres`), ou sqlx pour une API commune et des requêtes vérifiées à la compilation.
- rusqlite avec `bundled` : SQLite dans le binaire, rien à installer ; `execute`, `query_row` avec `.optional()`, `prepare` et `query_map`, `collect::<Result<Vec<_>, _>>()`.
- Le NULL se lit dans un `Option<T>` ; le lire dans un `T` est une erreur, pas une valeur par défaut.
- Une transaction se termine par `commit()` ou par son `Drop`, qui fait le `ROLLBACK` : tout `?` entre les deux annule.
- Les valeurs passent par `params!` ou `&[&a, &b]`, jamais par `format!` ; `?1` pour SQLite, `$1` pour PostgreSQL.
- La crate `postgres` vérifie les types strictement (`int4` dans `i32`, `int8` dans `i64`), `try_get` rend l'erreur, et `as_db_error().code()` donne le SQLSTATE.
- `tokio-postgres` pour l'async : la `Connection` se lance dans sa propre tâche ; deadpool ou le pool de sqlx dans un service, dimensionné contre `max_connections`.
- Un `impl TryFrom<&Row>` par struct pour projeter les lignes, un trait `Depot` pour cacher le moteur, SQLite en mémoire pour tester.
- Le DSN vient de l'environnement, ne s'affiche pas dans `--help`, et ne figure dans aucun message d'erreur.

---

← [13. HTTP et async : tokio, reqwest, axum](13-http-et-async.md) · [Sommaire](../README.md) · [15. Qualité, build, release, Docker](15-qualite-build-release.md) →
