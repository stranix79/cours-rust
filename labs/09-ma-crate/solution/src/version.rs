// Labo 09, solution : le module version (src/version.rs).
// Lancé par : (cd solution && cargo test version)   depuis labs/09-ma-crate
//
// Le type Version, sa lecture depuis du texte, son affichage et son ordre.
// Les fonctions utilitaires sont privées au module : invisibles depuis
// main.rs ou depuis tests/, mais testées par le module de tests en bas du
// fichier, qui y a accès parce qu'il est un enfant de ce module.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Une version sémantique : `MAJEUR.MINEUR.CORRECTIF[-PRÉVERSION][+BUILD]`.
///
/// L'égalité et l'ordre suivent la précédence semver : les métadonnées de
/// build sont ignorées, et une préversion passe avant la version finale.
// Clone mais pas Copy : deux String dedans. Pas de derive(PartialEq) : la
// version dérivée comparerait aussi `build`, ce qui contredirait Ord.
#[derive(Debug, Clone)]
pub struct Version {
    /// Change quand l'API casse.
    pub majeur: u64,
    /// Change quand une fonctionnalité arrive sans rien casser.
    pub mineur: u64,
    /// Change pour une correction de bug.
    pub correctif: u64,
    /// Les identifiants après le `-` (`beta.1`), sans le tiret.
    pub prerelease: Option<String>,
    /// Les identifiants après le `+` (`build.5`), sans le plus.
    pub build: Option<String>,
}

/// Ce qui peut être faux dans une version écrite à la main.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurVersion {
    /// Le texte est vide.
    Vide,
    /// Pas exactement trois nombres séparés par des points (le texte fautif).
    Forme(String),
    /// Une partie n'est pas un entier valide, ou commence par un zéro (`01`).
    Nombre(String),
    /// Un identifiant de préversion ou de build est vide ou contient un
    /// caractère interdit (seuls `[0-9A-Za-z-]` sont permis).
    Identifiant(String),
}

impl fmt::Display for ErreurVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurVersion::Vide => write!(f, "version vide"),
            ErreurVersion::Forme(s) => write!(f, "{s:?} : attendu MAJEUR.MINEUR.CORRECTIF"),
            ErreurVersion::Nombre(s) => write!(f, "{s:?} n'est pas un entier valide"),
            ErreurVersion::Identifiant(s) => write!(f, "identifiant {s:?} invalide"),
        }
    }
}

// Display + Debug + ce impl vide : ErreurVersion est une vraie erreur, que
// `?` sait convertir en Box<dyn Error> ou en anyhow::Error (chapitre 8).
impl std::error::Error for ErreurVersion {}

/// Lit un entier semver : des chiffres seulement, sans zéro en tête (« 0 »
/// est permis, « 01 » non). Privé : c'est un détail du module.
fn lire_nombre(s: &str) -> Result<u64, ErreurVersion> {
    // u64::from_str accepterait « +1 » : on vérifie d'abord qu'il n'y a que
    // des chiffres ASCII. is_empty d'abord, parce que all() est vrai sur vide.
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErreurVersion::Nombre(s.to_string()));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(ErreurVersion::Nombre(s.to_string()));
    }
    // Reste le débordement (plus de 20 chiffres) : parse le détecte.
    s.parse().map_err(|_| ErreurVersion::Nombre(s.to_string()))
}

/// Vérifie une liste d'identifiants séparés par des points (`beta.1`,
/// `build.5`). `numeriques_stricts` interdit les zéros en tête des
/// identifiants numériques, ce que semver exige pour la préversion seulement.
fn verifier_identifiants(s: &str, numeriques_stricts: bool) -> Result<(), ErreurVersion> {
    for id in s.split('.') {
        let permis = |c: char| c.is_ascii_alphanumeric() || c == '-';
        if id.is_empty() || !id.chars().all(permis) {
            return Err(ErreurVersion::Identifiant(id.to_string()));
        }
        let numerique = id.bytes().all(|b| b.is_ascii_digit());
        if numeriques_stricts && numerique && id.len() > 1 && id.starts_with('0') {
            return Err(ErreurVersion::Identifiant(id.to_string()));
        }
    }
    Ok(())
}

/// Compare deux identifiants de préversion selon semver : deux nombres se
/// comparent comme des nombres, un nombre passe avant un mot, deux mots se
/// comparent dans l'ordre ASCII.
fn comparer_identifiants(a: &str, b: &str) -> Ordering {
    // parse::<u64>().ok() rend Some pour un identifiant numérique : le match
    // sur la paire couvre les quatre cas d'un coup.
    match (a.parse::<u64>().ok(), b.parse::<u64>().ok()) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.cmp(b), // l'ordre de str est celui des octets
    }
}

impl Version {
    /// Lit une version semver complète. Les espaces autour et un `v` en tête
    /// (comme dans un tag git) sont tolérés.
    ///
    /// ```
    /// use semver_maison::Version;
    ///
    /// let v = Version::parse("1.2.3-beta.1+build.5").unwrap();
    /// assert_eq!((v.majeur, v.mineur, v.correctif), (1, 2, 3));
    /// assert_eq!(v.prerelease.as_deref(), Some("beta.1"));
    /// assert_eq!(v.build.as_deref(), Some("build.5"));
    /// assert_eq!(v.to_string(), "1.2.3-beta.1+build.5");
    ///
    /// assert!(Version::parse("1.2").is_err());
    /// ```
    pub fn parse(s: &str) -> Result<Version, ErreurVersion> {
        let texte = s.trim();
        // strip_prefix rend Option<&str> : unwrap_or(texte) garde le texte
        // tel quel s'il n'y avait pas de « v ». Aucune copie : des tranches.
        let texte = texte.strip_prefix('v').unwrap_or(texte);
        if texte.is_empty() {
            return Err(ErreurVersion::Vide);
        }
        // Le build d'abord : il est après le premier « + », et il peut
        // contenir des « - » qu'il ne faut pas prendre pour une préversion.
        let (reste, build) = match texte.split_once('+') {
            Some((r, b)) => (r, Some(b)),
            None => (texte, None),
        };
        // Puis la préversion : après le PREMIER « - » (« alpha-1 » est un
        // seul identifiant, avec un tiret dedans).
        let (coeur, prerelease) = match reste.split_once('-') {
            Some((c, p)) => (c, Some(p)),
            None => (reste, None),
        };

        let parties: Vec<&str> = coeur.split('.').collect();
        // Un motif de tranche : exactement trois éléments, nommés au passage.
        let [ma, mi, co] = parties.as_slice() else {
            return Err(ErreurVersion::Forme(coeur.to_string()));
        };
        if let Some(p) = prerelease {
            verifier_identifiants(p, true)?;
        }
        if let Some(b) = build {
            verifier_identifiants(b, false)?;
        }
        Ok(Version {
            // `?` renvoie l'ErreurVersion de lire_nombre telle quelle : même
            // type d'erreur, pas de conversion.
            majeur: lire_nombre(ma)?,
            mineur: lire_nombre(mi)?,
            correctif: lire_nombre(co)?,
            // Option<&str> → Option<String> : on copie les tranches pour que
            // la Version possède ses données et ne dépende plus de `s`.
            prerelease: prerelease.map(str::to_string),
            build: build.map(str::to_string),
        })
    }

    /// Le triplet seul, pratique pour comparer sans préversion.
    pub fn triplet(&self) -> (u64, u64, u64) {
        (self.majeur, self.mineur, self.correctif)
    }
}

/// Permet `"1.2.3".parse::<Version>()`, comme pour les nombres.
impl FromStr for Version {
    type Err = ErreurVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.majeur, self.mineur, self.correctif)?;
        // if let sur une référence : on n'emprunte que le temps d'écrire.
        if let Some(p) = &self.prerelease {
            write!(f, "-{p}")?;
        }
        if let Some(b) = &self.build {
            write!(f, "+{b}")?;
        }
        Ok(())
    }
}

impl Ord for Version {
    /// La précédence semver : le triplet, puis la préversion (absente = plus
    /// grande), identifiant par identifiant. Le build est ignoré.
    fn cmp(&self, autre: &Self) -> Ordering {
        // Les tuples se comparent champ par champ : c'est l'ordre
        // lexicographique qu'on veut. then_with ne calcule la suite que si
        // le triplet est égal.
        self.triplet().cmp(&autre.triplet()).then_with(|| {
            match (&self.prerelease, &autre.prerelease) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater, // 1.0.0 > 1.0.0-rc.1
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => {
                    // Identifiant par identifiant ; si tous les identifiants
                    // communs sont égaux, le plus court passe avant
                    // (alpha < alpha.1). zip s'arrête au plus court, la
                    // comparaison des longueurs règle la fin.
                    let ids_a: Vec<&str> = a.split('.').collect();
                    let ids_b: Vec<&str> = b.split('.').collect();
                    ids_a
                        .iter()
                        .zip(&ids_b)
                        .map(|(x, y)| comparer_identifiants(x, y))
                        .find(|o| o.is_ne())
                        .unwrap_or_else(|| ids_a.len().cmp(&ids_b.len()))
                }
            }
        })
    }
}

// PartialOrd et PartialEq délèguent à cmp : une seule source de vérité.
// Si on dérivait PartialEq, 1.0.0+a et 1.0.0+b seraient différents pour ==
// et égaux pour cmp, et un BTreeSet ou un sort se comporteraient bizarrement.
impl PartialOrd for Version {
    fn partial_cmp(&self, autre: &Self) -> Option<Ordering> {
        Some(self.cmp(autre))
    }
}

impl PartialEq for Version {
    fn eq(&self, autre: &Self) -> bool {
        self.cmp(autre) == Ordering::Equal
    }
}

impl Eq for Version {}

// Les tests unitaires : compilés seulement par `cargo test` (cfg(test)),
// dans un module enfant qui voit les fonctions privées grâce à `use super::*`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lire_nombre_refuse_les_zeros_en_tete() {
        assert_eq!(lire_nombre("0"), Ok(0));
        assert_eq!(lire_nombre("42"), Ok(42));
        assert!(lire_nombre("01").is_err());
        assert!(lire_nombre("+1").is_err());
        assert!(lire_nombre("").is_err());
        assert!(lire_nombre("99999999999999999999999").is_err()); // déborde u64
    }

    #[test]
    fn identifiants_numeriques_avant_les_mots() {
        assert_eq!(comparer_identifiants("2", "10"), Ordering::Less);
        assert_eq!(comparer_identifiants("10", "alpha"), Ordering::Less);
        assert_eq!(comparer_identifiants("beta", "alpha"), Ordering::Greater);
        assert_eq!(comparer_identifiants("rc", "rc"), Ordering::Equal);
    }

    #[test]
    fn identifiants_valides() {
        assert!(verifier_identifiants("beta.1", true).is_ok());
        assert!(verifier_identifiants("alpha-1", true).is_ok());
        assert!(verifier_identifiants("beta..1", true).is_err()); // vide
        assert!(verifier_identifiants("beta.01", true).is_err()); // zéro en tête
        assert!(verifier_identifiants("build.007", false).is_ok()); // permis en build
        assert!(verifier_identifiants("été", false).is_err()); // pas ASCII
    }
}
