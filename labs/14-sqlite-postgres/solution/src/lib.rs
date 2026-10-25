// Labo 14, solution : un inventaire derrière un trait Depot, avec deux
// implémentations, SQLite (rusqlite) et PostgreSQL (crate postgres).
// Lancé par : (cd solution && cargo test), ou via src/main.rs :
//   (cd solution && cargo run -- --dsn sqlite:inventaire.db lister)
//
// Le trait est le contrat ; les deux structs le remplissent chacune avec sa
// crate. Le reste du programme (main.rs, les tests) ne parle qu'au trait, et
// ne sait pas quel moteur est derrière.

use std::cell::RefCell; // prêter le Client postgres en &mut depuis un &self
use std::path::Path; // le chemin du fichier SQLite

use anyhow::{Context, bail}; // contexte sur les erreurs, et bail! pour en créer une
use rusqlite::{OptionalExtension, params}; // .optional() et la macro params!

/// Le schéma, embarqué dans le binaire à la compilation. include_str! lit le
/// fichier relativement à CE fichier source et en fait un &'static str : rien
/// à copier sur le serveur, le binaire porte son schéma.
const MIGRATION_001: &str = include_str!("../migrations/001.sql");

/// Un article de l'inventaire, tel que les deux moteurs le rendent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub nom: String,
    pub quantite: i64,
}

/// Ce que le programme attend d'un dépôt, quel que soit le moteur.
///
/// `&mut self` pour les écritures : une transaction rusqlite ou postgres
/// emprunte la connexion de façon exclusive, et le compilateur le vérifie.
/// `&self` pour les lectures : on peut lire depuis une référence partagée.
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

/// La règle métier commune aux deux moteurs : on n'ajoute ni ne retire une
/// quantité nulle ou négative. Écrite une fois, appelée par les deux.
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

/// Projeter une ligne SQLite vers notre type. TryFrom plutôt qu'une fonction
/// libre : c'est le trait standard de la conversion qui peut échouer, et
/// `Article::try_from(row)` se lit tout seul. Les colonnes sont lues par nom,
/// pas par position : un SELECT réordonné ne casse rien.
impl TryFrom<&rusqlite::Row<'_>> for Article {
    type Error = rusqlite::Error;

    fn try_from(ligne: &rusqlite::Row<'_>) -> Result<Self, Self::Error> {
        Ok(Article {
            nom: ligne.get("nom")?,
            quantite: ligne.get("quantite")?,
        })
    }
}

impl DepotSqlite {
    /// Ouvre (ou crée) la base dans le fichier `chemin` et applique le schéma.
    ///
    /// # Errors
    /// Fichier impossible à ouvrir, ou migration qui échoue.
    pub fn nouveau(chemin: impl AsRef<Path>) -> anyhow::Result<Self> {
        let chemin = chemin.as_ref();
        let conn = rusqlite::Connection::open(chemin)
            .with_context(|| format!("ouverture de {}", chemin.display()))?;
        Self::migrer(conn)
    }

    /// Une base en mémoire, neuve, qui disparaît avec la struct : parfaite
    /// pour les tests (quelques millisecondes, rien sur le disque).
    ///
    /// # Errors
    /// Migration qui échoue.
    pub fn en_memoire() -> anyhow::Result<Self> {
        Self::migrer(rusqlite::Connection::open_in_memory()?)
    }

    /// Applique le schéma embarqué. execute_batch accepte plusieurs
    /// instructions séparées par des points-virgules (execute n'en prend
    /// qu'une), sans paramètres : c'est fait pour le DDL.
    fn migrer(conn: rusqlite::Connection) -> anyhow::Result<Self> {
        conn.execute_batch(MIGRATION_001).context("migration 001")?;
        Ok(DepotSqlite { conn })
    }

    /// Vide la table (pour les tests).
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
        verifier_quantite(quantite)?;
        // Un UPSERT : une seule instruction, donc atomique, sans transaction
        // explicite. `excluded` désigne la ligne qu'on tentait d'insérer.
        // ?1 et ?2 sont des paramètres liés : la valeur part à côté du SQL,
        // jamais dedans. Un nom comme `x'); DROP TABLE articles; --` reste
        // un nom (la fin du scénario des tests le vérifie).
        self.conn.execute(
            "INSERT INTO articles (nom, quantite) VALUES (?1, ?2)
             ON CONFLICT (nom) DO UPDATE SET quantite = articles.quantite + excluded.quantite",
            params![nom, quantite],
        )?;
        Ok(())
    }

    fn retirer(&mut self, nom: &str, quantite: i64) -> anyhow::Result<i64> {
        verifier_quantite(quantite)?;
        // transaction() emprunte la connexion en &mut : tant que `tx` vit, on
        // ne peut rien faire d'autre avec self.conn. Si on sort de la
        // fonction sans commit (un ? ou un bail!), le Drop de `tx` fait le
        // ROLLBACK. C'est le `defer tx.Rollback()` de Go, fait par le
        // compilateur.
        let tx = self.conn.transaction()?;
        // query_row rend une erreur QueryReturnedNoRows quand il n'y a pas de
        // ligne ; .optional() la transforme en Ok(None). C'est le
        // sql.ErrNoRows de Go, rangé dans le type.
        let actuel: Option<i64> = tx
            .query_row(
                "SELECT quantite FROM articles WHERE nom = ?1",
                params![nom],
                |ligne| ligne.get(0),
            )
            .optional()?;
        let Some(actuel) = actuel else {
            bail!("article inconnu : {nom}");
        };
        if actuel < quantite {
            bail!("stock insuffisant pour {nom} : {actuel} en stock, {quantite} demandés");
        }
        let reste = actuel - quantite;
        tx.execute(
            "UPDATE articles SET quantite = ?1 WHERE nom = ?2",
            params![reste, nom],
        )?;
        // commit() consomme la transaction : impossible de l'utiliser après.
        tx.commit()?;
        Ok(reste)
    }

    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>> {
        let stock = self
            .conn
            .query_row(
                "SELECT quantite FROM articles WHERE nom = ?1",
                params![nom],
                |ligne| ligne.get(0),
            )
            .optional()?;
        Ok(stock)
    }

    fn lister(&self) -> anyhow::Result<Vec<Article>> {
        // prepare compile la requête une fois ; query_map l'exécute et rend
        // un itérateur de Result<Article>, une par ligne.
        let mut requete = self
            .conn
            .prepare("SELECT nom, quantite FROM articles ORDER BY nom")?;
        // collect sur des Result : le premier Err arrête tout et devient le
        // résultat. Pas de rows.Err() à oublier comme en Go.
        let articles = requete
            .query_map([], |ligne| Article::try_from(ligne))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }
}

// ------------------------------------------------------------ PostgreSQL

/// Le dépôt PostgreSQL : un Client de la crate postgres, une connexion.
///
/// Le Client demande `&mut self` même pour lire (il écrit la requête sur la
/// socket). Notre trait promet `&self` pour stock() et lister() : on range
/// donc le Client dans un RefCell (chapitre 10), qui prête un `&mut` à
/// l'exécution à partir d'un `&self`. Le dépôt n'est pas partagé entre
/// threads, le RefCell suffit ; à plusieurs threads, ce serait un Mutex.
pub struct DepotPostgres {
    client: RefCell<postgres::Client>,
}

/// La même projection, pour une ligne PostgreSQL. try_get vérifie le type :
/// lire un BIGINT dans un i32 est une erreur claire, pas une troncature.
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
    /// Se connecte et applique le schéma.
    ///
    /// NoTls : connexion en clair, bien pour 127.0.0.1 ou un réseau Docker.
    /// Ailleurs, il faut une crate de TLS en plus (postgres-native-tls ou
    /// tokio-postgres-rustls).
    ///
    /// # Errors
    /// Connexion refusée, authentification, ou migration qui échoue.
    pub fn connecter(dsn: &str) -> anyhow::Result<Self> {
        // Le contexte ne cite pas le DSN : il peut contenir un mot de passe,
        // et les messages d'erreur finissent dans des journaux.
        let mut client =
            postgres::Client::connect(dsn, postgres::NoTls).context("connexion à PostgreSQL")?;
        client
            .batch_execute(MIGRATION_001)
            .context("migration 001")?;
        Ok(DepotPostgres {
            client: RefCell::new(client),
        })
    }

    /// Vide la table (pour les tests).
    ///
    /// # Errors
    /// Erreur de la base.
    pub fn vider(&mut self) -> anyhow::Result<()> {
        // get_mut() : on a déjà &mut self, donc un accès exclusif prouvé à la
        // compilation ; pas besoin du contrôle à l'exécution de borrow_mut().
        self.client.get_mut().execute("DELETE FROM articles", &[])?;
        Ok(())
    }
}

impl Depot for DepotPostgres {
    fn ajouter(&mut self, nom: &str, quantite: i64) -> anyhow::Result<()> {
        verifier_quantite(quantite)?;
        // Même SQL, autre syntaxe de paramètres : $1, $2 pour PostgreSQL.
        // Les paramètres sont une slice de références à des valeurs qui
        // implémentent ToSql : &nom est un &&str, &quantite un &i64.
        self.client.get_mut().execute(
            "INSERT INTO articles (nom, quantite) VALUES ($1, $2)
             ON CONFLICT (nom) DO UPDATE SET quantite = articles.quantite + excluded.quantite",
            &[&nom, &quantite],
        )?;
        Ok(())
    }

    fn retirer(&mut self, nom: &str, quantite: i64) -> anyhow::Result<i64> {
        verifier_quantite(quantite)?;
        // Même patron que SQLite : la transaction emprunte le client, son
        // Drop fait le ROLLBACK si on n'arrive pas au commit.
        let mut tx = self.client.get_mut().transaction()?;
        // FOR UPDATE verrouille la ligne jusqu'à la fin de la transaction :
        // deux retraits simultanés depuis deux programmes ne peuvent pas lire
        // le même stock et le décrémenter chacun de leur côté. SQLite n'en a
        // pas besoin : il n'accepte qu'un écrivain à la fois.
        let ligne = tx.query_opt(
            "SELECT quantite FROM articles WHERE nom = $1 FOR UPDATE",
            &[&nom],
        )?;
        let Some(ligne) = ligne else {
            bail!("article inconnu : {nom}");
        };
        // get() panique si le type ne correspond pas ; try_get rend une
        // erreur. BIGINT donne un i64, c'est le contrat du schéma.
        let actuel: i64 = ligne.try_get(0)?;
        if actuel < quantite {
            bail!("stock insuffisant pour {nom} : {actuel} en stock, {quantite} demandés");
        }
        let reste = actuel - quantite;
        tx.execute(
            "UPDATE articles SET quantite = $1 WHERE nom = $2",
            &[&reste, &nom],
        )?;
        tx.commit()?;
        Ok(reste)
    }

    fn stock(&self, nom: &str) -> anyhow::Result<Option<i64>> {
        // borrow_mut() prête le Client en &mut le temps de l'instruction. Un
        // second borrow_mut() pendant celui-ci paniquerait : ça n'arrive pas,
        // chaque méthode emprunte et rend dans la même ligne.
        let ligne = self
            .client
            .borrow_mut()
            .query_opt("SELECT quantite FROM articles WHERE nom = $1", &[&nom])?;
        // Option<Row> vers Option<i64> : map donne Option<Result<i64>>, et
        // transpose() le retourne en Result<Option<i64>>, que ? déballe.
        Ok(ligne.map(|l| l.try_get::<_, i64>(0)).transpose()?)
    }

    fn lister(&self) -> anyhow::Result<Vec<Article>> {
        let lignes = self
            .client
            .borrow_mut()
            .query("SELECT nom, quantite FROM articles ORDER BY nom", &[])?;
        // Même collect sur des Result que pour SQLite.
        let articles = lignes
            .iter()
            .map(Article::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(articles)
    }
}

/// Ouvre le bon dépôt selon le DSN :
///
/// - `sqlite::memory:` : SQLite en mémoire ;
/// - `sqlite:chemin` : SQLite dans un fichier ;
/// - `postgres://...` ou `postgresql://...` : PostgreSQL.
///
/// `Box<dyn Depot>` : le type exact n'est connu qu'à l'exécution, on rend
/// donc un objet trait (chapitre 7), alloué sur le tas.
///
/// # Errors
/// DSN non reconnu, ou ouverture du dépôt qui échoue.
pub fn ouvrir(dsn: &str) -> anyhow::Result<Box<dyn Depot>> {
    if dsn == "sqlite::memory:" {
        Ok(Box::new(DepotSqlite::en_memoire()?))
    } else if let Some(chemin) = dsn.strip_prefix("sqlite:") {
        Ok(Box::new(DepotSqlite::nouveau(chemin)?))
    } else if dsn.starts_with("postgres://") || dsn.starts_with("postgresql://") {
        Ok(Box::new(DepotPostgres::connecter(dsn)?))
    } else {
        // Toujours pas le DSN dans le message, pour la même raison.
        bail!("DSN non reconnu : il doit commencer par sqlite: ou postgres://")
    }
}
