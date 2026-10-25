// Labo 14 : un inventaire derrière un trait Depot, avec deux implémentations,
// SQLite (rusqlite) et PostgreSQL (crate postgres).
// Lancé par : cargo test (tests/depot.rs), ou via src/main.rs :
//   cargo run -- --dsn sqlite:inventaire.db lister
//
// Le trait, les types et les signatures sont fournis. Les corps todo!() sont
// à écrire dans l'ordre des TODO : d'abord SQLite (TODO 1 à 6, testés à
// chaque `cargo test`), puis PostgreSQL (TODO 7, testé si INVENTAIRE_DSN est
// définie, voir le README).

// Le squelette a des paramètres et des imports pas encore utilisés : on fait
// taire ces avertissements le temps du labo. Retire cette ligne à la fin.
#![allow(unused_variables, unused_imports, unused_mut, dead_code)]

use std::cell::RefCell;
use std::path::Path;

use anyhow::{Context, bail};
use rusqlite::{OptionalExtension, params};

/// Le schéma, embarqué dans le binaire à la compilation (lis migrations/001.sql).
const MIGRATION_001: &str = include_str!("../migrations/001.sql");

/// Un article de l'inventaire, tel que les deux moteurs le rendent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub nom: String,
    pub quantite: i64,
}

/// Ce que le programme attend d'un dépôt, quel que soit le moteur.
pub trait Depot {
    /// Ajoute `quantite` au stock de `nom`, en créant l'article s'il manque.
    ///
    /// # Errors
    /// Quantité nulle ou négative, ou erreur de la base.
    fn ajouter(&mut self, nom: &str, quantite: i64) -> anyhow::Result<()>;

    /// Retire `quantite` du stock de `nom` et rend le stock restant. Tout ou
    /// rien : en cas d'erreur, le stock n'a pas bougé.
    ///
    /// # Errors
    /// Quantité invalide, article inconnu, stock insuffisant, erreur de la base.
    fn retirer(&mut self, nom: &str, quantite: i64) -> anyhow::Result<i64>;

    /// Le stock de `nom`, ou `None` si l'article n'existe pas.
    ///
    /// # Errors
    /// Erreur de la base.
    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>>;

    /// Tous les articles, triés par nom.
    ///
    /// # Errors
    /// Erreur de la base.
    fn lister(&self) -> anyhow::Result<Vec<Article>>;
}

/// Fourni : la règle commune aux deux moteurs. Appelle-la au début de
/// ajouter() et de retirer().
fn verifier_quantite(quantite: i64) -> anyhow::Result<()> {
    if quantite <= 0 {
        bail!("quantité invalide : {quantite} (elle doit être positive)");
    }
    Ok(())
}

// ---------------------------------------------------------------- SQLite

/// Le dépôt SQLite : une seule connexion, possédée par la struct.
pub struct DepotSqlite {
    conn: rusqlite::Connection,
}

impl TryFrom<&rusqlite::Row<'_>> for Article {
    type Error = rusqlite::Error;

    fn try_from(ligne: &rusqlite::Row<'_>) -> Result<Self, Self::Error> {
        // TODO 5 (b) : Ok(Article { nom: ligne.get("nom")?, quantite: ligne.get("quantite")? })
        todo!("TODO 5")
    }
}

impl DepotSqlite {
    /// Fourni : ouvre (ou crée) la base dans le fichier `chemin`.
    ///
    /// # Errors
    /// Fichier impossible à ouvrir, ou migration qui échoue.
    pub fn nouveau(chemin: impl AsRef<Path>) -> anyhow::Result<Self> {
        let chemin = chemin.as_ref();
        let conn = rusqlite::Connection::open(chemin)
            .with_context(|| format!("ouverture de {}", chemin.display()))?;
        Self::migrer(conn)
    }

    /// Fourni : une base en mémoire, neuve, qui disparaît avec la struct.
    ///
    /// # Errors
    /// Migration qui échoue.
    pub fn en_memoire() -> anyhow::Result<Self> {
        Self::migrer(rusqlite::Connection::open_in_memory()?)
    }

    fn migrer(conn: rusqlite::Connection) -> anyhow::Result<Self> {
        // TODO 1 : applique MIGRATION_001 avec conn.execute_batch(...) (avec
        // un .context("migration 001")?), puis rends Ok(DepotSqlite { conn }).
        todo!("TODO 1")
    }

    /// Fourni : vide la table (pour les tests).
    ///
    /// # Errors
    /// Erreur de la base.
    pub fn vider(&mut self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM articles", [])?;
        Ok(())
    }
}

impl Depot for DepotSqlite {
    fn ajouter(&mut self, nom: &str, quantite: i64) -> anyhow::Result<()> {
        // TODO 2 : verifier_quantite(quantite)?, puis un seul execute :
        //   INSERT INTO articles (nom, quantite) VALUES (?1, ?2)
        //   ON CONFLICT (nom) DO UPDATE SET quantite = articles.quantite + excluded.quantite
        // avec params![nom, quantite]. Jamais de format!() dans le SQL.
        todo!("TODO 2")
    }

    fn retirer(&mut self, nom: &str, quantite: i64) -> anyhow::Result<i64> {
        // TODO 3 : verifier_quantite, puis let tx = self.conn.transaction()?;
        // lis la quantité avec tx.query_row(...).optional()? ; None ->
        // bail!("article inconnu : {nom}") ; pas assez ->
        // bail!("stock insuffisant pour {nom} : {actuel} en stock, {quantite} demandés") ;
        // sinon UPDATE, tx.commit()?, et rends le reste. Un bail! avant le
        // commit annule la transaction tout seul (le Drop de tx).
        todo!("TODO 3")
    }

    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>> {
        // TODO 4 : self.conn.query_row("SELECT quantite FROM articles WHERE nom = ?1",
        // params![nom], |ligne| ligne.get(0)).optional()?, dans un Ok(...).
        todo!("TODO 4")
    }

    fn lister(&self) -> anyhow::Result<Vec<Article>> {
        // TODO 5 (a) : self.conn.prepare("SELECT nom, quantite FROM articles ORDER BY nom")?,
        // puis .query_map([], |ligne| Article::try_from(ligne))? et
        // .collect::<Result<Vec<_>, _>>()?. Écris aussi le TryFrom plus haut.
        todo!("TODO 5")
    }
}

// ------------------------------------------------------------ PostgreSQL

/// Le dépôt PostgreSQL. Le Client demande `&mut self` même pour lire : il
/// est rangé dans un RefCell pour que stock() et lister(), qui n'ont que
/// `&self`, puissent l'emprunter en `&mut` avec `self.client.borrow_mut()`.
/// Dans ajouter() et retirer(), qui ont `&mut self`, prends `self.client.get_mut()`.
pub struct DepotPostgres {
    client: RefCell<postgres::Client>,
}

/// Fourni : la projection d'une ligne PostgreSQL.
impl TryFrom<&postgres::Row> for Article {
    type Error = postgres::Error;

    fn try_from(ligne: &postgres::Row) -> Result<Self, Self::Error> {
        Ok(Article {
            nom: ligne.try_get("nom")?,
            quantite: ligne.try_get("quantite")?,
        })
    }
}

impl DepotPostgres {
    /// Fourni : se connecte (en clair, NoTls) et applique le schéma.
    ///
    /// # Errors
    /// Connexion refusée, authentification, ou migration qui échoue.
    pub fn connecter(dsn: &str) -> anyhow::Result<Self> {
        let mut client =
            postgres::Client::connect(dsn, postgres::NoTls).context("connexion à PostgreSQL")?;
        client
            .batch_execute(MIGRATION_001)
            .context("migration 001")?;
        Ok(DepotPostgres {
            client: RefCell::new(client),
        })
    }

    /// Fourni : vide la table (pour les tests).
    ///
    /// # Errors
    /// Erreur de la base.
    pub fn vider(&mut self) -> anyhow::Result<()> {
        self.client.get_mut().execute("DELETE FROM articles", &[])?;
        Ok(())
    }
}

// TODO 7 : transpose les quatre méthodes SQLite pour PostgreSQL. Les
// différences : les paramètres s'écrivent $1, $2 et se passent en
// &[&nom, &quantite] ; query_opt rend une Option<Row> ; query rend un
// Vec<Row> ; try_get(0) lit une colonne. Dans retirer, ajoute FOR UPDATE au
// SELECT : la ligne reste verrouillée jusqu'au commit.
impl Depot for DepotPostgres {
    fn ajouter(&mut self, nom: &str, quantite: i64) -> anyhow::Result<()> {
        todo!("TODO 7")
    }

    fn retirer(&mut self, nom: &str, quantite: i64) -> anyhow::Result<i64> {
        todo!("TODO 7")
    }

    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>> {
        todo!("TODO 7")
    }

    fn lister(&self) -> anyhow::Result<Vec<Article>> {
        todo!("TODO 7")
    }
}

/// Ouvre le bon dépôt selon le DSN :
///
/// - `sqlite::memory:` : SQLite en mémoire ;
/// - `sqlite:chemin` : SQLite dans un fichier ;
/// - `postgres://...` ou `postgresql://...` : PostgreSQL.
///
/// # Errors
/// DSN non reconnu, ou ouverture du dépôt qui échoue.
pub fn ouvrir(dsn: &str) -> anyhow::Result<Box<dyn Depot>> {
    // TODO 6 : un if / else if sur dsn (==, strip_prefix("sqlite:"),
    // starts_with("postgres://")), chaque branche rend Ok(Box::new(...)).
    // Sinon bail!("DSN non reconnu : il doit commencer par sqlite: ou postgres://")
    // et surtout pas le DSN lui-même : il peut contenir un mot de passe.
    todo!("TODO 6")
}
