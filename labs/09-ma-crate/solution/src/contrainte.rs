// Labo 09, solution : le module contrainte (src/contrainte.rs).
// Lancé par : (cd solution && cargo test contrainte)   depuis labs/09-ma-crate
//
// Une contrainte de version à la Cargo (`^1.2`, `~1.2.3`, `>=1.0, <2.0`,
// `=1.2.3`, `*`) est traduite UNE fois, à la lecture, en une liste de bornes
// simples (`>=`, `>`, `<`, `<=`, `=` sur une version complète). Vérifier une
// version revient ensuite à tester chaque borne : tout le travail difficile
// est dans parse, satisfait est trivial.

use std::fmt;

// `crate::` part de la racine de la crate (lib.rs) : c'est le chemin absolu.
// `super::version::Version` marcherait aussi, c'est le chemin relatif.
use crate::version::{ErreurVersion, Version};

/// Ce qui peut être faux dans une contrainte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurContrainte {
    /// La contrainte (ou un de ses morceaux entre virgules) est vide.
    Vide,
    /// Une version partielle illisible après l'opérateur (le texte fautif).
    Partielle(String),
    /// Une version complète illisible.
    Version(ErreurVersion),
}

impl fmt::Display for ErreurContrainte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurContrainte::Vide => write!(f, "contrainte vide"),
            ErreurContrainte::Partielle(s) => write!(f, "version partielle {s:?} illisible"),
            ErreurContrainte::Version(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ErreurContrainte {
    // source() donne l'erreur d'origine : un rapport d'erreur (anyhow, au
    // chapitre 8) peut remonter la chaîne.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErreurContrainte::Version(e) => Some(e),
            _ => None,
        }
    }
}

// Avec ce From, `?` sur un Result<_, ErreurVersion> dans une fonction qui
// rend ErreurContrainte convertit tout seul. C'est ce que thiserror
// écrirait pour toi avec #[from].
impl From<ErreurVersion> for ErreurContrainte {
    fn from(e: ErreurVersion) -> Self {
        ErreurContrainte::Version(e)
    }
}

/// Les cinq comparaisons élémentaires. Privé : un détail de représentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Egal,
    Sup,
    SupEgal,
    Inf,
    InfEgal,
}

/// Une contrainte de version : un ET de bornes simples.
#[derive(Debug, Clone)]
pub struct Contrainte {
    // Privé : l'utilisateur ne construit une Contrainte que par parse, donc
    // toujours valide. Vide = `*`, tout passe.
    bornes: Vec<(Op, Version)>,
}

/// Une version partielle : `1`, `1.2`, `1.2.3`, `1.2.3-beta`. Les parties
/// absentes sont None.
struct Partielle {
    majeur: u64,
    mineur: Option<u64>,
    correctif: Option<u64>,
    prerelease: Option<String>,
}

/// Construit une Version complète, sans build (inutile pour comparer).
fn version(majeur: u64, mineur: u64, correctif: u64, prerelease: Option<String>) -> Version {
    Version {
        majeur,
        mineur,
        correctif,
        prerelease,
        build: None,
    }
}

/// Lit une version partielle. Une version complète passe par Version::parse
/// pour bénéficier de toutes ses vérifications.
fn lire_partielle(s: &str) -> Result<Partielle, ErreurContrainte> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ErreurContrainte::Vide);
    }
    // Trois parties ou plus (ou un « - », « + ») : c'est une version
    // complète, on délègue. `?` convertit ErreurVersion grâce au From.
    if s.split('.').count() >= 3 || s.contains(['-', '+']) {
        let v = Version::parse(s)?;
        return Ok(Partielle {
            majeur: v.majeur,
            mineur: Some(v.mineur),
            correctif: Some(v.correctif),
            prerelease: v.prerelease,
        });
    }
    // Une ou deux parties : des nombres simples.
    let mut nombres = s.split('.').map(|p| {
        p.parse::<u64>()
            .map_err(|_| ErreurContrainte::Partielle(s.to_string()))
    });
    // next() rend Option<Result<u64, _>> : transpose() en fait un
    // Result<Option<u64>, _>, que `?` sait déballer.
    let majeur = nombres.next().transpose()?.ok_or(ErreurContrainte::Vide)?;
    let mineur = nombres.next().transpose()?;
    Ok(Partielle {
        majeur,
        mineur,
        correctif: None,
        prerelease: None,
    })
}

/// Traduit un comparateur (`^1.2`, `>=1.0`, `~1.2.3`…) en bornes simples,
/// ajoutées à `bornes`. Les règles sont celles de Cargo.
fn traduire(comparateur: &str, bornes: &mut Vec<(Op, Version)>) -> Result<(), ErreurContrainte> {
    let c = comparateur.trim();
    if c == "*" {
        return Ok(()); // aucune borne : tout passe
    }
    // Les opérateurs à deux caractères AVANT ceux à un seul, sinon « >=1.0 »
    // serait lu « > » suivi de « =1.0 ». Pas d'opérateur : `^`, comme dans
    // un Cargo.toml où « 1.2 » veut dire « ^1.2 ».
    let (op, reste) = [">=", "<=", ">", "<", "=", "^", "~"]
        .iter()
        .find_map(|op| c.strip_prefix(op).map(|r| (*op, r)))
        .unwrap_or(("^", c));
    let p = lire_partielle(reste)?;
    let (ma, mi, co) = (p.majeur, p.mineur.unwrap_or(0), p.correctif.unwrap_or(0));
    // La borne basse, parties absentes remplies de zéros.
    let bas = version(ma, mi, co, p.prerelease.clone());

    match op {
        ">=" => bornes.push((Op::SupEgal, bas)),
        "<" => bornes.push((Op::Inf, bas)),
        ">" => match (p.mineur, p.correctif) {
            (Some(_), Some(_)) => bornes.push((Op::Sup, bas)),
            // « >1.2 » : au-delà de toute la série 1.2.x, donc >= 1.3.0.
            (Some(mi), None) => bornes.push((Op::SupEgal, version(ma, mi + 1, 0, None))),
            (None, _) => bornes.push((Op::SupEgal, version(ma + 1, 0, 0, None))),
        },
        "<=" => match (p.mineur, p.correctif) {
            (Some(_), Some(_)) => bornes.push((Op::InfEgal, bas)),
            // « <=1.2 » : toute la série 1.2.x comprise, donc < 1.3.0.
            (Some(mi), None) => bornes.push((Op::Inf, version(ma, mi + 1, 0, None))),
            (None, _) => bornes.push((Op::Inf, version(ma + 1, 0, 0, None))),
        },
        "=" => match (p.mineur, p.correctif) {
            (Some(_), Some(_)) => bornes.push((Op::Egal, bas)),
            // « =1.2 » : n'importe quel 1.2.x.
            (Some(mi), None) => {
                bornes.push((Op::SupEgal, bas));
                bornes.push((Op::Inf, version(ma, mi + 1, 0, None)));
            }
            (None, _) => {
                bornes.push((Op::SupEgal, bas));
                bornes.push((Op::Inf, version(ma + 1, 0, 0, None)));
            }
        },
        "~" => {
            // ~1.2.3 et ~1.2 : même majeur.mineur ; ~1 : même majeur.
            let haut = match p.mineur {
                Some(mi) => version(ma, mi + 1, 0, None),
                None => version(ma + 1, 0, 0, None),
            };
            bornes.push((Op::SupEgal, bas));
            bornes.push((Op::Inf, haut));
        }
        _ => {
            // Le caret : on ne change pas le premier chiffre non nul, parce
            // que semver dit qu'avant 1.0.0, chaque mineur peut casser l'API.
            let haut = match (ma, p.mineur, p.correctif) {
                (0, Some(0), Some(co)) => version(0, 0, co + 1, None), // ^0.0.3
                (0, Some(mi), _) => version(0, mi + 1, 0, None),       // ^0.2, ^0.0
                _ => version(ma + 1, 0, 0, None),                      // ^1.2, ^0
            };
            bornes.push((Op::SupEgal, bas));
            bornes.push((Op::Inf, haut));
        }
    }
    Ok(())
}

impl Contrainte {
    /// Lit une contrainte à la Cargo. Plusieurs comparateurs séparés par des
    /// virgules doivent TOUS être vrais ; une version sans opérateur vaut
    /// `^` (comme dans un `Cargo.toml`).
    pub fn parse(s: &str) -> Result<Contrainte, ErreurContrainte> {
        if s.trim().is_empty() {
            return Err(ErreurContrainte::Vide);
        }
        let mut bornes = Vec::new();
        // Chaque morceau est traduit ; le premier qui échoue arrête tout
        // grâce à `?`. « 1.2, » donne un morceau vide : erreur Vide.
        for comparateur in s.split(',') {
            traduire(comparateur, &mut bornes)?;
        }
        Ok(Contrainte { bornes })
    }

    /// Dit si `v` satisfait la contrainte.
    ///
    /// Une préversion ne satisfait une contrainte que si un de ses
    /// comparateurs porte lui-même une préversion sur le même triplet : on ne
    /// veut pas qu'un `^1.2` attrape `1.3.0-alpha` par surprise.
    ///
    /// ```
    /// use semver_maison::{Contrainte, Version};
    ///
    /// let v = |s| Version::parse(s).unwrap();
    /// let caret = Contrainte::parse("^1.2").unwrap();
    /// assert!(caret.satisfait(&v("1.2.0")));
    /// assert!(caret.satisfait(&v("1.9.3")));
    /// assert!(!caret.satisfait(&v("2.0.0")));
    /// assert!(!caret.satisfait(&v("1.3.0-alpha")));
    ///
    /// let plage = Contrainte::parse(">=1.0, <2.0").unwrap();
    /// assert!(plage.satisfait(&v("1.99.0")));
    /// assert!(!plage.satisfait(&v("0.9.0")));
    /// ```
    pub fn satisfait(&self, v: &Version) -> bool {
        // all() sur une liste vide est vrai : « * » laisse tout passer.
        let dans_les_bornes = self.bornes.iter().all(|(op, borne)| match op {
            Op::Egal => v == borne,
            Op::Sup => v > borne,
            Op::SupEgal => v >= borne,
            Op::Inf => v < borne,
            Op::InfEgal => v <= borne,
        });
        // La règle des préversions : une version finale passe toujours ce
        // filtre ; une préversion seulement si une borne en a une au même
        // triplet (« >=1.2.3-beta » accepte 1.2.3-rc.1, pas 1.2.4-rc.1).
        let preversion_permise = v.prerelease.is_none()
            || self
                .bornes
                .iter()
                .any(|(_, b)| b.prerelease.is_some() && b.triplet() == v.triplet());
        dans_les_bornes && preversion_permise
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Accès aux privés : on vérifie la traduction elle-même, ce que les
    // tests d'intégration (qui ne voient que satisfait) ne peuvent pas faire.
    fn bornes(s: &str) -> Vec<(Op, String)> {
        Contrainte::parse(s)
            .unwrap()
            .bornes
            .into_iter()
            .map(|(op, v)| (op, v.to_string()))
            .collect()
    }

    #[test]
    fn caret_traduit_en_deux_bornes() {
        let attendu = vec![
            (Op::SupEgal, "1.2.0".to_string()),
            (Op::Inf, "2.0.0".to_string()),
        ];
        assert_eq!(bornes("^1.2"), attendu);
        assert_eq!(bornes("1.2"), attendu, "sans opérateur, c'est un caret");
    }

    #[test]
    fn caret_avant_un_zero_point() {
        assert_eq!(bornes("^0.2.3")[1], (Op::Inf, "0.3.0".to_string()));
        assert_eq!(bornes("^0.0.3")[1], (Op::Inf, "0.0.4".to_string()));
    }

    #[test]
    fn etoile_sans_borne() {
        assert!(bornes("*").is_empty());
    }

    #[test]
    fn partielle_illisible() {
        assert!(matches!(
            lire_partielle("1.x"),
            Err(ErreurContrainte::Partielle(_))
        ));
        assert!(matches!(
            lire_partielle("1.2.x"),
            Err(ErreurContrainte::Version(_))
        ));
    }
}
