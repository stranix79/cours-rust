// Labo 16 : depot.rs, l'historique des mesures dans SQLite.
//
// Le dépôt du labo 14 en plus petit : une connexion rusqlite possédée par la
// struct, le schéma embarqué avec include_str!, des paramètres liés partout.
// Il n'y a qu'un moteur, donc pas de trait : on en ajoutera un le jour où
// PostgreSQL arrive (c'est une piste de la version 2, dans le README).

use std::path::Path; // le chemin du fichier SQLite

use anyhow::Context; // contexte sur les erreurs d'ouverture et de migration
use rusqlite::{OptionalExtension, params}; // .optional() et params!

use crate::sonde::Mesure; // ce qu'on stocke

/// Le schéma, dans le binaire.
const MIGRATION_001: &str = include_str!("../migrations/001.sql");

/// L'historique des mesures. `Connection` est `Send` mais pas `Sync` : on peut
/// la déplacer d'un thread à l'autre, pas la partager. lib.rs la range donc
/// dans un `Mutex`, qui rend l'ensemble partageable entre les tâches tokio.
pub struct Depot {
    conn: rusqlite::Connection,
}

impl Depot {
    /// Ouvre (ou crée) la base. `:memory:` donne une base jetable.
    ///
    /// # Errors
    /// Fichier impossible à ouvrir, ou migration qui échoue.
    pub fn nouveau(chemin: impl AsRef<Path>) -> anyhow::Result<Self> {
        let chemin = chemin.as_ref();
        let conn = rusqlite::Connection::open(chemin)
            .with_context(|| format!("ouverture de {}", chemin.display()))?;
        Self::migrer(conn)
    }

    /// Une base en mémoire, neuve : pour les tests.
    ///
    /// # Errors
    /// Migration qui échoue.
    pub fn en_memoire() -> anyhow::Result<Self> {
        Self::migrer(rusqlite::Connection::open_in_memory()?)
    }

    fn migrer(conn: rusqlite::Connection) -> anyhow::Result<Self> {
        conn.execute_batch(MIGRATION_001).context("migration 001")?;
        Ok(Depot { conn })
    }

    /// Enregistre les mesures d'un tour, toutes ou aucune.
    ///
    /// Une transaction pour tout le tour, et pas seulement pour la cohérence :
    /// SQLite écrit sur le disque à chaque COMMIT, et cinquante INSERT dans une
    /// transaction coûtent un seul fsync au lieu de cinquante.
    ///
    /// # Errors
    /// Erreur de la base.
    pub fn enregistrer(&mut self, mesures: &[Mesure]) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        {
            // prepare_cached : la requête est compilée une fois et réutilisée
            // pour chaque ligne. Le bloc borne la vie de `requete`, qui
            // emprunte `tx` : il faut la rendre avant le commit.
            let mut requete = tx.prepare_cached(
                "INSERT INTO mesures (cible, instant, ok, duree_ms, erreur)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for m in mesures {
                // Un Option<String> devient NULL quand il vaut None.
                requete.execute(params![m.cible, m.instant, m.ok, m.duree_ms, m.erreur])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Les `n` mesures les plus récentes, de toutes les cibles ou d'une
    /// seule, de la plus récente à la plus ancienne.
    ///
    /// # Errors
    /// Erreur de la base.
    pub fn dernieres(&self, cible: Option<&str>, n: usize) -> anyhow::Result<Vec<Mesure>> {
        // `?1 IS NULL OR cible = ?1` : une seule requête pour les deux cas,
        // avec ou sans filtre, au lieu de deux chaînes SQL à maintenir. Et
        // toujours un paramètre : jamais de format!() dans du SQL.
        let mut requete = self.conn.prepare_cached(
            "SELECT cible, instant, ok, duree_ms, erreur FROM mesures
             WHERE ?1 IS NULL OR cible = ?1
             ORDER BY instant DESC, id DESC
             LIMIT ?2",
        )?;
        // SQLite veut un i64 pour LIMIT ; un n absurde est plafonné.
        let limite = i64::try_from(n).unwrap_or(i64::MAX);
        let mesures = requete
            .query_map(params![cible, limite], |ligne| {
                Ok(Mesure {
                    cible: ligne.get("cible")?,
                    instant: ligne.get("instant")?,
                    ok: ligne.get("ok")?,
                    duree_ms: ligne.get("duree_ms")?,
                    erreur: ligne.get("erreur")?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(mesures)
    }

    /// La part des mesures réussies de `cible` depuis `depuis` (secondes
    /// Unix, comprises), entre 0.0 et 1.0, ou `None` s'il n'y a aucune mesure.
    ///
    /// # Errors
    /// Erreur de la base.
    pub fn taux_disponibilite(&self, cible: &str, depuis: i64) -> anyhow::Result<Option<f64>> {
        // TODO 2 : une seule requête, SELECT AVG(ok) FROM mesures WHERE cible = ?1
        // AND instant >= ?2, avec params![cible, depuis]. AVG sur des 0 et des
        // 1 donne la proportion de 1 ; sur zéro ligne, il rend NULL. Lis la
        // colonne dans un Option<f64> (ligne.get(0)) avec query_row : NULL
        // devient None, et « pas de mesure » n'est pas « 0 % ».
        todo!("TODO 2")
    }
}
