// Labo 09 : le module version (src/version.rs).
// Lancé par : cargo test version   (les tests dont le nom contient « version »)
//
// Le type Version, sa lecture depuis du texte, son affichage et son ordre.
// Les fonctions utilitaires sont privées au module : invisibles depuis
// main.rs ou depuis tests/, mais testées par le module de tests en bas du
// fichier (ne le modifie pas), qui y a accès parce qu'il est un enfant de
// ce module.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Une version sémantique : `MAJEUR.MINEUR.CORRECTIF[-PRÉVERSION][+BUILD]`.
///
/// L'égalité et l'ordre suivent la précédence semver : les métadonnées de
/// build sont ignorées, et une préversion passe avant la version finale.
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

impl std::error::Error for ErreurVersion {}

/// Lit un entier semver : des chiffres seulement, sans zéro en tête (« 0 »
/// est permis, « 01 » non). Privé : c'est un détail du module.
// Le allow ciblé évite l'avertissement « jamais utilisée » tant que parse
// est un todo!() ; retire-le quand parse appelle cette fonction.
#[allow(dead_code)]
fn lire_nombre(s: &str) -> Result<u64, ErreurVersion> {
    // TODO 1 : refuse (Err(ErreurVersion::Nombre(s.to_string()))) un texte
    // vide, un texte qui contient autre chose que des chiffres ASCII
    // (s.bytes().all(|b| b.is_ascii_digit())), et un nombre de plus d'un
    // chiffre qui commence par '0'. Puis s.parse() avec map_err pour le
    // débordement. Attention : "+1".parse::<u64>() réussit, d'où le test
    // des chiffres AVANT parse.
    todo!("TODO 1 : lire_nombre({s:?})")
}

/// Vérifie une liste d'identifiants séparés par des points (`beta.1`,
/// `build.5`). `numeriques_stricts` interdit les zéros en tête des
/// identifiants numériques, ce que semver exige pour la préversion seulement.
/// Fournie : lis-la, tu en auras besoin dans parse.
#[allow(dead_code)] // à retirer quand parse appelle cette fonction
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
#[allow(dead_code)] // à retirer quand cmp appelle cette fonction
fn comparer_identifiants(a: &str, b: &str) -> Ordering {
    // TODO 3 : match sur la paire (a.parse::<u64>().ok(), b.parse::<u64>().ok()) :
    // (Some(x), Some(y)) => x.cmp(&y), (Some(_), None) => Ordering::Less,
    // (None, Some(_)) => Ordering::Greater, (None, None) => a.cmp(b).
    todo!("TODO 3 : comparer_identifiants({a:?}, {b:?})")
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
        // TODO 2 : dans l'ordre, sans copier (des &str jusqu'au bout) :
        // - s.trim(), puis strip_prefix('v').unwrap_or(texte) ;
        // - vide : Err(ErreurVersion::Vide) ;
        // - split_once('+') sépare le build (à droite), puis split_once('-')
        //   sur ce qui reste sépare la préversion (le PREMIER tiret) ;
        // - le cœur split('.') doit donner exactement trois parties :
        //   `let [ma, mi, co] = parties.as_slice() else { return Err(Forme) };`
        // - verifier_identifiants(pre, true)? et (build, false)? s'il y en a ;
        // - lire_nombre(ma)? pour chaque partie, et .map(str::to_string)
        //   pour passer d'Option<&str> à Option<String>.
        todo!("TODO 2 : Version::parse({s:?})")
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
        // TODO 5 : write!(f, "{}.{}.{}", ...)?, puis "-{p}" si la préversion
        // existe (if let Some(p) = &self.prerelease) et "+{b}" pour le build.
        // Termine par Ok(()).
        let _ = f;
        todo!("TODO 5 : Display pour Version {:?}", self.triplet())
    }
}

impl Ord for Version {
    /// La précédence semver : le triplet, puis la préversion (absente = plus
    /// grande), identifiant par identifiant. Le build est ignoré.
    fn cmp(&self, autre: &Self) -> Ordering {
        // TODO 4 : self.triplet().cmp(&autre.triplet()).then_with(|| ...) où
        // la fermeture compare les préversions : (None, None) Equal,
        // (None, Some) Greater (1.0.0 > 1.0.0-rc.1), (Some, None) Less, et
        // (Some(a), Some(b)) identifiant par identifiant : split('.') des
        // deux côtés, zip, comparer_identifiants, le premier résultat non
        // Equal (.find(|o| o.is_ne())), sinon le plus court passe avant
        // (comparaison des longueurs).
        todo!("TODO 4 : {:?} cmp {:?}", self.triplet(), autre.triplet())
    }
}

// Fournis : PartialOrd et PartialEq délèguent à cmp, une seule source de
// vérité. Un derive(PartialEq) comparerait aussi le build et contredirait Ord.
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

// Les tests unitaires : ne les modifie pas. Ils voient les fonctions privées
// grâce à `use super::*`.
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
