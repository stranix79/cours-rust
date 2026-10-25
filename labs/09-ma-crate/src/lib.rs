// Labo 09 : la racine de la bibliothèque semver_maison.
// Lancé par : cargo test   (fourni complet : lis-le, n'y touche pas)
//
// Ce fichier ne contient presque pas de code : il déclare les modules et
// choisit ce qui est public. C'est le rôle de lib.rs dans une crate bien
// rangée : le sommaire, pas le contenu.

//! Versions sémantiques et contraintes de version, à la manière de Cargo.
//!
//! La crate lit des versions de la forme `MAJEUR.MINEUR.CORRECTIF`, avec une
//! préversion et des métadonnées de build optionnelles
//! (`1.2.3-beta.1+build.5`, voir <https://semver.org>), les compare selon la
//! précédence semver, et vérifie des contraintes comme celles d'un
//! `Cargo.toml` : `^1.2`, `~1.2.3`, `>=1.0, <2.0`, `=1.2.3`, `*`.
//!
//! ```
//! use semver_maison::{Contrainte, Version};
//!
//! let v = Version::parse("1.5.0").unwrap();
//! let c = Contrainte::parse("^1.2").unwrap();
//! assert!(c.satisfait(&v));
//! assert!(Version::parse("1.0.0-rc.1").unwrap() < Version::parse("1.0.0").unwrap());
//! ```

// Un avertissement pour chaque élément public sans doc : la doc fait partie
// de l'API. `cargo doc --open` affiche ce qu'on écrit ici avec `//!` et `///`.
#![warn(missing_docs)]

// Les deux modules sont PRIVÉS (pas de `pub mod`) : leurs chemins internes
// (semver_maison::version::Version) ne font pas partie de l'API. On choisit
// ce qui sort avec `pub use`, et on reste libre de réorganiser les fichiers
// sans casser personne. Chaque `mod x;` va chercher src/x.rs.
mod contrainte;
mod version;

// Les re-exports : l'utilisateur écrit `semver_maison::Version`, court.
pub use contrainte::{Contrainte, ErreurContrainte};
pub use version::{ErreurVersion, Version};
