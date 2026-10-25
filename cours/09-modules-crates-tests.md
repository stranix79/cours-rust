# 9. Modules, crates, Cargo, tests, documentation

*Rust de zéro à la prod : chapitre 9 sur 16.* ← [8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic](08-erreurs.md) · [Sommaire](../README.md) · [10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe](10-sous-le-capot.md) →

Jusqu'ici, chaque exemple tenait dans un `main.rs`. Un vrai outil a une bibliothèque, un ou deux exécutables, des modules internes, des tests, une doc, et un `Cargo.toml` qui fixe ses dépendances. Ce chapitre t'apprend à **ranger** : où mettre les fichiers, ce qu'un autre module peut voir, comment Cargo choisit les versions, et comment écrire les trois sortes de tests de Rust. Comme en Go, tout est dans la boîte : `cargo test` et `cargo doc` ne demandent rien d'installer, et tout le monde les utilise de la même façon.

Fil rouge : une petite crate `portscan`, la cousine Rust de celle du chapitre 9 du cours Go, avec une bibliothèque qui transforme `"22,80,8000-8010"` en liste de ports et un exécutable qui s'en sert. Tout ce qui suit a été lancé dessus.

### 9.1 Paquet, crate, module : trois mots pour trois choses

Rust a trois niveaux là où Go en a deux, et le vocabulaire déroute au début.

- Le **paquet** (*package*) est ce que Cargo gère : un dossier avec un `Cargo.toml`. Il a un nom, une version, des dépendances.
- Une **crate** est une unité de compilation : ce que `rustc` compile d'un coup, en partant d'un fichier racine. Un paquet contient au plus une crate bibliothèque (racine `src/lib.rs`) et autant de crates binaires qu'il veut (racine `src/main.rs`, plus un fichier par binaire dans `src/bin/`).
- Un **module** est un espace de noms à l'intérieur d'une crate. Une crate est un arbre de modules, dont la racine est `lib.rs` ou `main.rs`.

Voici l'arbre de `portscan` :

```
portscan/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs            la racine de la bibliothèque : déclare les modules, choisit l'API
│   ├── ports.rs          le module ports : parse, MAX, ErreurPorts
│   ├── ports/
│   │   └── plage.rs      le sous-module ports::plage : un détail d'implémentation
│   ├── main.rs           l'exécutable portscan : lit l'argument, appelle la bibliothèque
│   └── bin/
│       └── compter.rs    un second exécutable, compter
└── tests/
    └── parse.rs          les tests d'intégration
```

La différence qui compte avec Go : **un module n'est pas un dossier, et un fichier n'est pas un module tant qu'on ne l'a pas déclaré**. Un fichier `src/ports.rs` qui traîne sans `mod ports;` quelque part n'est pas compilé du tout. Rien n'est découvert automatiquement, sauf `src/lib.rs`, `src/main.rs`, `src/bin/*.rs`, `tests/*.rs`, `benches/*.rs` et `examples/*.rs`, que Cargo connaît par convention.

### 9.2 `mod`, `pub` et les chemins

Le `lib.rs` de `portscan` ne contient presque pas de code :

```rust
//! Analyse des listes de ports au format `"22,80,8000-8010"`.
//!
//! ```
//! let ports = portscan::parse("22,8000-8002").unwrap();
//! assert_eq!(ports, [22, 8000, 8001, 8002]);
//! ```

#![warn(missing_docs)]

pub mod ports;

pub use ports::{ErreurPorts, parse};
```

`pub mod ports;` dit trois choses : il existe un module `ports`, son code est dans `src/ports.rs`, et il est public. Le `pub use` qui suit est un **re-export** : l'utilisateur écrit `portscan::parse` au lieu de `portscan::ports::parse`. Tu choisis ainsi l'API de ta crate indépendamment du rangement de tes fichiers, et tu peux réorganiser l'intérieur sans casser personne. C'est le rôle de `lib.rs` : le sommaire, pas le contenu.

La règle de visibilité de Rust tient en une phrase : **tout est privé par défaut, et privé veut dire visible dans le module où c'est déclaré et dans ses descendants**. `pub` ouvre à tout le monde. Entre les deux, deux crans :

- `pub(crate)` : visible partout dans la crate, invisible de l'extérieur. C'est le `internal/` de Go, à l'échelle d'une fonction au lieu d'un dossier.
- `pub(super)` : visible dans le module parent. Pratique pour un sous-module qui rend service à son parent et à personne d'autre.

Le module `ports` déclare un sous-module privé, dans `src/ports/plage.rs` :

```rust
//! Un élément : un port seul ou une plage `debut-fin`.

use super::{ErreurPorts, MAX};

/// Les bornes d'un élément : `"80"` donne (80, 80), `"1-3"` donne (1, 3).
pub(super) fn bornes(elem: &str) -> Result<(u16, u16), ErreurPorts> {
    if elem.is_empty() {
        return Err(ErreurPorts::Vide);
    }
    let (a, b) = elem.split_once('-').unwrap_or((elem, elem));
    let (a, b) = (un_port(a)?, un_port(b)?);
    if a > b {
        return Err(ErreurPorts::Inversee(a, b));
    }
    Ok((a, b))
}

fn un_port(s: &str) -> Result<u16, ErreurPorts> {
    match s.trim().parse::<u16>() {
        Ok(p) if (1..=MAX).contains(&p) => Ok(p),
        _ => Err(ErreurPorts::Invalide(s.to_string())),
    }
}
```

`super::` part du module parent, comme `..` dans un chemin de fichier. Les deux autres points de départ : `crate::` part de la racine de la crate (le chemin absolu, `crate::ports::MAX`), et `self::` du module courant (rarement utile, sauf pour lever une ambiguïté dans un `use`). Un nom de crate externe (`std::`, `serde::`, `portscan::`) part de cette crate. Et `use` ne fait qu'une chose : il crée un raccourci local vers un chemin, il ne charge rien.

Depuis `main.rs`, qui est une **autre crate** (le binaire) et voit la bibliothèque comme n'importe quel client, essaie d'atteindre `bornes` :

```rust
fn main() {
    let b = portscan::ports::plage::bornes("1-3");
    println!("{b:?}");
}
```

```
$ cargo build
error[E0603]: module `plage` is private
 --> src/main.rs:2:30
  |
2 |     let b = portscan::ports::plage::bornes("1-3");
  |                              ^^^^^  ------ function `bornes` is not publicly re-exported
  |                              |
  |                              private module
  |
note: the module `plage` is defined here
 --> src/ports.rs:3:1
  |
3 | mod plage;
  | ^^^^^^^^^
```

C'est la fermeture du module `plage` qui bloque, avant même la visibilité de `bornes`. Le compilateur te dit exactement où il est déclaré. La correction est de ne pas le faire : `main` appelle `portscan::parse`, l'API, et `bornes` reste un détail qu'on a le droit de changer demain.

**Venant de Go :** la majuscule exportée devient `pub`, explicite, mot par mot. Il n'y a pas de visibilité « paquet » implicite : deux fichiers d'un même dossier ne se voient pas d'office, ils doivent être des modules et se référencer par chemin. En échange, tu as `pub(crate)` et `pub(super)`, plus fins que la seule frontière de paquet. Et les champs d'une struct sont privés un par un : une struct `pub` aux champs privés ne se construit que par ses fonctions, ce qui garantit ses invariants (le labo 09 s'en sert pour `Contrainte`).

**Venant du C :** `pub` est ce que tu mettrais dans le `.h`, et le privé par défaut est un `static` sur tout ce qui n'est pas exporté. Sauf que tu n'écris pas d'en-tête : le compilateur lit la crate elle-même, et une signature ne peut pas diverger entre le `.h` et le `.c`.

### 9.3 Fichiers et dossiers

`mod ports;` dans `lib.rs` va chercher `src/ports.rs`. `mod plage;` dans `ports.rs` va chercher `src/ports/plage.rs` : le dossier porte le nom du module parent. C'est toute la règle, et le compilateur te la rappelle si tu oublies le fichier :

```
$ cargo build
error[E0583]: file not found for module `absent`
  --> src/lib.rs:13:1
   |
13 | mod absent;
   | ^^^^^^^^^^^
   |
   = help: to create the module `absent`, create file "src/absent.rs" or "src/absent/mod.rs"
```

Le message mentionne la seconde forme, `src/absent/mod.rs` : c'est l'ancienne convention (édition 2015), où un module à sous-modules vivait dans `nom/mod.rs`. Elle marche toujours, et tu la verras dans beaucoup de crates anciennes, mais elle donne dix onglets qui s'appellent tous `mod.rs` dans ton éditeur. Utilise `nom.rs` plus `nom/sous.rs`, c'est la forme moderne.

On peut aussi écrire un module en ligne, `mod tests { ... }` dans le fichier lui-même : c'est ce qu'on fait pour les tests unitaires (9.8).

**Venant de Python :** `mod ports;` joue le rôle de l'`__init__.py` qui importe ses sous-modules, sauf qu'il est obligatoire : un fichier non déclaré n'existe pas. Et il n'y a pas d'import circulaire possible au sens Python : la crate est compilée d'un bloc, les modules se voient tous par leurs chemins, l'ordre de déclaration ne compte pas.

### 9.4 Le prélude, et ce que `use` importe vraiment

Tu écris `String`, `Vec`, `Option`, `Some`, `Ok`, `Box`, `.clone()` et `.into()` sans aucun `use` depuis le chapitre 2. C'est le **prélude** : une courte liste de noms que le compilateur importe dans chaque module (`std::prelude::rust_2024`). Tout le reste se nomme : `use std::collections::HashMap;`, `use std::fmt;`, `use std::io::{self, BufRead};` (`self` ici importe le module `io` lui-même en plus de `BufRead`).

Deux usages à connaître. Pour les **traits**, `use` est indispensable : une méthode de trait n'est appelable que si le trait est en portée. `reader.lines()` ne compile pas sans `use std::io::BufRead;`, et le compilateur te le suggère. Et beaucoup de crates offrent leur propre prélude, à importer en bloc : `use rayon::prelude::*;` (chapitre 11). L'import avec `*` est réservé à ces préludes et aux modules de tests (`use super::*;`) ; ailleurs, nomme ce que tu importes, pour qu'on sache d'où vient chaque nom.

### 9.5 Une bibliothèque et des binaires dans le même paquet

`src/lib.rs` et `src/main.rs` côte à côte, c'est la structure à adopter **dès le premier jour** pour tout outil. Le binaire est minuscule : il lit les arguments, appelle la bibliothèque, affiche, choisit le code de sortie.

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    let spec = std::env::args().nth(1).unwrap_or_default();
    match portscan::parse(&spec) {
        Ok(ports) => {
            println!("{} port(s) : {ports:?}", ports.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("portscan : {e}");
            ExitCode::from(2)
        }
    }
}
```

```
$ cargo run -q -- '22,80,8000-8002'
5 port(s) : [22, 80, 8000, 8001, 8002]
$ cargo run -q -- '90-80' ; echo "code $?"
portscan : plage inversée 90-80
code 2
```

Pourquoi ce découpage : la bibliothèque se teste sans lancer de processus, les tests d'intégration (9.9) ne peuvent importer qu'une bibliothèque, et le jour où tu veux une API HTTP au-dessus du même code (chapitre 13), il n'y a rien à déplacer. ripgrep est construit ainsi, uv et ruff aussi.

D'autres exécutables ? Un fichier par binaire dans `src/bin/`. `src/bin/compter.rs` devient le binaire `compter`, qui utilise la même bibliothèque :

```rust
// Un second binaire : src/bin/compter.rs devient l'exécutable `compter`.
use portscan::ports::MAX;

fn main() {
    let spec = std::env::args().nth(1).unwrap_or_default();
    let n = portscan::parse(&spec).map(|p| p.len()).unwrap_or(0);
    println!("{n} port(s) sur {MAX}");
}
```

```
$ cargo run -q -- 22
error: `cargo run` could not determine which binary to run. Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: compter, portscan
$ cargo run -q --bin compter -- 1-1024
1024 port(s) sur 65535
```

Par défaut, la bibliothèque porte le nom du paquet (tirets changés en soulignés) et le binaire principal aussi. Pour les renommer, `[lib] name = "..."` et `[[bin]] name = "..."` dans `Cargo.toml` : c'est ce que font les labos de ce cours depuis le labo 01, pour avoir un binaire `hello` dans un paquet `lab01-hello`.

### 9.6 `Cargo.toml`, versions, features, `Cargo.lock`

Ajoute une dépendance avec `cargo add` plutôt qu'à la main : il cherche la dernière version, l'écrit, et te montre les features.

```
$ cargo add serde --features derive
    Updating crates.io index
      Adding serde v1.0.229 to dependencies
             Features:
             + derive
             + serde_derive
             + std
             - alloc
             - rc
             - unstable
     Locking 7 packages to latest Rust 1.93.1 compatible versions
$ cargo add anyhow@1.0
      Adding anyhow v1.0 to dependencies
$ sed -n '/\[dependencies\]/,$p' Cargo.toml
[dependencies]
anyhow = "1.0"
serde = { version = "1.0.229", features = ["derive"] }
```

Trois choses à lire là-dedans.

**La version est une contrainte, pas une version.** `"1.0.229"` veut dire `^1.0.229` : n'importe quelle version `>= 1.0.229` et `< 2.0.0`. C'est le *caret*, implicite dans un `Cargo.toml`, et il repose sur la promesse de semver : tant que le majeur ne bouge pas, rien ne casse. Avant 1.0, la règle se décale d'un cran : `"0.8"` veut dire `>= 0.8.0, < 0.9.0`, parce qu'en 0.x chaque mineur a le droit de casser. Les autres formes : `"~1.2.3"` (correctifs seulement), `"=1.2.3"` (exactement), `">=1.2, <1.5"`, `"*"` (n'importe laquelle, refusé sur crates.io). Le labo 09 te fait écrire ces règles toi-même.

**Les features** sont des options de compilation d'une crate : du code qui n'est compilé que si on le demande. `serde` sans `derive` n'a pas les macros `#[derive(Serialize)]`. `tokio` sans features ne contient presque rien (chapitre 13). Les `+` et `-` de `cargo add` montrent ce qui est actif. Une feature ne s'enlève jamais : si deux dépendances demandent `serde`, l'une avec `derive` et l'autre sans, les deux reçoivent `derive`. Ce sont des **ajouts**, conçus pour s'additionner.

**`Cargo.lock`** enregistre la version exacte de chaque crate de l'arbre et son empreinte :

```
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
```

Tant que le fichier existe, `cargo build` utilise ces versions-là, sur ta machine comme en CI, même si `serde 1.0.230` sort demain. `cargo update` le met à jour dans les limites des contraintes (`cargo update -p serde` pour une seule crate), et `cargo tree` montre l'arbre complet, avec `cargo tree -i syn` pour savoir qui tire une crate. Quand le commiter ? La réponse a changé en 2023 : **toujours**, binaire comme bibliothèque. Pour un binaire, c'est évident : c'est la garantie qu'on livre ce qu'on a testé. Pour une bibliothèque, le fichier est ignoré par ceux qui dépendent d'elle (chacun résout avec ses propres contraintes), mais il rend ta propre CI reproductible.

**Venant de Go :** `Cargo.lock` est `go.sum` plus la version exacte que `go.mod` porte déjà. La grosse différence est l'algorithme. Go prend la **plus petite** version qui satisfait tout le monde (*minimal version selection*) et n'a donc pas besoin de verrou pour être reproductible. Cargo prend la **plus récente** compatible au moment de la résolution, et le verrou fige ce choix. Deuxième différence : en Go, `v2` change le chemin d'import ; en Rust, deux majeurs d'une même crate peuvent coexister dans un arbre sans rien changer au code (`cargo tree -d` liste ces doublons, qu'on aime éviter parce qu'ils doublent le temps de compilation). Et `cargo add` est `go get`, `cargo update` est `go get -u`, `cargo tree` est `go mod graph` en lisible.

**Venant de Python :** `Cargo.toml` est `pyproject.toml`, `Cargo.lock` est le `uv.lock` (ou le `poetry.lock`), et il n'y a pas d'environnement virtuel : chaque projet compile ses dépendances dans son `target/`, et le cache `~/.cargo/registry` garde les sources téléchargées une seule fois pour toute la machine.

### 9.7 Les espaces de travail

Un espace de travail (*workspace*) regroupe plusieurs paquets qui partagent un `Cargo.lock` et un dossier `target/`. Ce cours en est un, et son `Cargo.toml` racine tient en trois lignes utiles :

```
[workspace]
resolver = "3"
members = ["labs/*", "labs/*/solution"]
```

Chaque labo reste un paquet indépendant avec son propre `Cargo.toml`, mais les dépendances (serde, tokio, axum) ne sont compilées qu'une fois pour tous, et `cargo test` lancé dans `labs/09-ma-crate/` ne construit que ce labo. `resolver = "3"` est la version de l'algorithme de résolution, celle de l'édition 2024 : il préfère, pour chaque dépendance, la dernière version compatible avec la version de Rust que tu déclares (`rust-version` dans `Cargo.toml`), au lieu de la dernière tout court. Pour un vrai projet, l'espace de travail sert quand le code grossit : ripgrep a une dizaine de crates `grep-*` et un binaire `rg`, chacune compilée et testée à part, et `cargo test --workspace` lance tout. Une dépendance interne s'écrit avec un chemin : `grep-regex = { path = "../regex" }`.

### 9.8 crates.io et docs.rs

**crates.io** est l'index central des crates publiées, l'équivalent de PyPI. **docs.rs** construit automatiquement la doc de chaque version de chaque crate publiée, avec le même outil que `cargo doc`. C'est là que tu passeras le plus de temps, alors apprends à lire une page :

- En haut, le nom, la version (un menu permet de choisir une version plus ancienne, celle de ton `Cargo.lock`) et le **Feature flags** : la liste des features et ce que chacune active. Si une fonction de la doc n'existe pas chez toi, c'est presque toujours une feature manquante, et la doc l'indique par une étiquette « Available on crate feature X only ».
- La page d'accueil de la crate est son `//!` de `lib.rs` : un exemple d'usage, en général le meilleur point d'entrée.
- À gauche, les modules, structs, enums, traits, fonctions et macros. Sur la page d'un type : ses méthodes propres, puis **Trait Implementations** (ce que le type sait faire : `Display`, `From`, `Iterator`...), puis **Auto Trait Implementations** (`Send`, `Sync`, chapitre 11) et **Blanket Implementations**. Quand tu te demandes « est-ce que je peux faire `?` avec cette erreur dans un `anyhow::Result` », la réponse est dans la liste des traits.
- Le bouton **Source** à côté de chaque élément ouvre le code. Lire la source d'une fonction de la bibliothèque standard est le meilleur moyen de comprendre ce qu'elle fait vraiment, et c'est souvent court.

Pour choisir une crate : le nombre de téléchargements récents, la date de la dernière version, les dépendances inverses, et l'avis de [blessed.rs](https://blessed.rs), une liste tenue à jour des crates que la communauté considère comme le choix par défaut pour chaque besoin.

### 9.9 Tests unitaires : `#[cfg(test)] mod tests`

En Rust, un test unitaire vit **dans le fichier du code qu'il teste**, dans un module `tests` en bas :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_port() {
        assert_eq!(parse("22"), Ok(vec![22]));
    }

    #[test]
    fn plage_inversee() {
        assert_eq!(parse("90-80"), Err(ErreurPorts::Inversee(90, 80)));
    }

    #[test]
    fn bornes_est_prive_mais_visible() {
        // plage::bornes est pub(super) : visible dans ports, donc ici.
        assert_eq!(plage::bornes("1-3"), Ok((1, 3)));
    }
}
```

`#[cfg(test)]` : ce module n'est compilé que pour `cargo test`, il n'existe pas dans le binaire livré. `use super::*;` importe tout le module parent, privés compris. Et comme `tests` est un **enfant** de `ports`, il voit tout ce que `ports` voit : la règle de visibilité de 9.2, sans exception spéciale pour les tests. C'est la grosse différence avec Go et Python : tester une fonction privée est normal, et encouragé quand elle a de la logique.

`assert_eq!(gauche, droite)` compare et, en cas d'échec, affiche les deux valeurs avec `Debug` (d'où le `#[derive(Debug, PartialEq)]` sur `ErreurPorts`). Un troisième argument optionnel est un message formaté : `assert_eq!(n, 3, "parse({spec:?})")`. `assert!(condition)` et `assert_ne!` complètent la panoplie. Un test échoue s'il panique, c'est tout le mécanisme.

```
$ cargo test
     Running unittests src/lib.rs (target/debug/deps/portscan-690ddcecd0d0c07c)
running 3 tests
test ports::tests::bornes_est_prive_mais_visible ... ok
test ports::tests::plage_inversee ... ok
test ports::tests::un_port ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/main.rs (target/debug/deps/portscan-f3c5ba711b6a1456)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/parse.rs (target/debug/deps/parse-d8c282f2433acb75)
running 2 tests
test erreurs ... ok
test melange ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests portscan
running 2 tests
test src/ports.rs - ports::parse (line 35) ... ok
test src/lib.rs - (line 3) ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Quatre sections pour un paquet : les tests unitaires de la bibliothèque, ceux du binaire (il peut en avoir, le labo 09 en met dans `main.rs`), chaque fichier de `tests/`, et les tests de doc. Les tests d'un même binaire tournent **en parallèle**, dans des threads : deux tests qui écrivent le même fichier ou la même variable globale se marchent dessus. Donne à chacun son dossier temporaire (le labo 12 montre comment), ou sérialise-les avec un `Mutex` statique (le labo 10 le fait pour un compteur global), ou en dernier recours `cargo test -- --test-threads=1`.

**Venant de Go :** `go test` et `cargo test` ont la même philosophie (dans la boîte, découverte par convention, pas de framework), mais Rust a `assert_eq!` là où Go te fait écrire `if got != want { t.Errorf(...) }`. Le message d'échec est moins personnalisé et plus régulier : `left`, `right`, les deux valeurs. Il n'y a pas de `t.Run` pour les sous-tests : on boucle sur un tableau de cas et on met le cas dans le message de l'assertion, comme dans tous les tests des labos. Et pas de cache des résultats : `cargo test` relance tout, mais ne recompile que ce qui a changé.

**Venant de Python :** c'est pytest, avec la découverte par attribut (`#[test]`) au lieu du préfixe `test_`, et `assert_eq!` qui fait ce que l'introspection d'`assert` fait chez pytest. Les fixtures n'existent pas : une fonction utilitaire dans le module de tests, et un type avec `Drop` pour le nettoyage (le `DossierTemp` du labo 12), font le même travail sans magie.

### 9.10 Tests d'intégration : le dossier `tests/`

Chaque fichier de `tests/` est compilé comme une **crate à part**, qui dépend de ta bibliothèque exactement comme le ferait un utilisateur : il ne voit que l'API publique.

```rust
use portscan::{ErreurPorts, parse};

#[test]
fn melange() {
    assert_eq!(parse("22,8000-8001").unwrap(), [22, 8000, 8001]);
}

#[test]
fn erreurs() {
    assert_eq!(parse(""), Err(ErreurPorts::Vide));
    assert_eq!(parse("ssh"), Err(ErreurPorts::Invalide("ssh".into())));
    assert_eq!(parse("70000"), Err(ErreurPorts::Invalide("70000".into())));
}
```

Pas de `#[cfg(test)]` ici : tout le dossier n'existe que pour les tests. Les deux sortes se complètent. Les tests unitaires vérifient les rouages (une fonction privée, un cas limite d'un détail), les tests d'intégration vérifient le contrat : si un test de `tests/` casse, un utilisateur casse aussi. Ce sont les tests d'intégration que les labos de ce cours fournissent, identiques entre squelette et solution, parce que c'est le contrat que ta version doit remplir.

Une crate qui n'a qu'un `main.rs` ne peut pas avoir de tests d'intégration : il n'y a rien à importer. Encore une raison de toujours séparer `lib.rs`.

### 9.11 Tests de doc : un exemple qui ne peut pas mentir

Un commentaire `///` au-dessus d'un élément est sa documentation, en Markdown. Un bloc de code dedans est un **test**, compilé et exécuté par `cargo test` :

```rust
/// Transforme `"22,80,8000-8002"` en `[22, 80, 8000, 8001, 8002]`.
///
/// Les espaces autour des éléments sont ignorés.
///
/// ```
/// use portscan::parse;
///
/// assert_eq!(parse("22, 80").unwrap(), [22, 80]);
/// assert_eq!(parse("8000-8002").unwrap(), [8000, 8001, 8002]);
/// assert!(parse("90-80").is_err());
/// ```
pub fn parse(spec: &str) -> Result<Vec<u16>, ErreurPorts> {
    let mut ports = Vec::new();
    for elem in spec.split(',') {
        let (debut, fin) = plage::bornes(elem.trim())?;
        ports.extend(debut..=fin);
    }
    Ok(ports)
}
```

Le bloc est compilé comme un petit programme extérieur à la crate (d'où le `use portscan::parse;`), avec un `fn main` ajouté autour. `//!` documente l'élément qui le contient : en tête de `lib.rs`, c'est la page d'accueil de la crate, et son exemple est testé aussi. Fais mentir la doc : remplace `[22, 80]` par `[22, 81]`.

```
$ cargo test --doc
running 2 tests
test src/lib.rs - (line 3) ... ok
test src/ports.rs - ports::parse (line 35) ... FAILED
failures:
---- src/ports.rs - ports::parse (line 35) stdout ----
Test executable failed (exit status: 101).
stderr:
thread 'main' (55360431) panicked at /var/folders/…/doctest_bundle_2024.rs:17:1:
assertion `left == right` failed
  left: [22, 80]
 right: [22, 81]
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
error: doctest failed, to rerun pass `--doc`
```

C'est l'équivalent exact de l'`Example` avec `// Output:` du Go et du `doctest` de Python : une doc qui casse le build le jour où elle devient fausse. Quelques variantes du bloc : ` ```no_run ` compile sans exécuter (un exemple qui ouvre une connexion réseau), ` ```ignore ` ne compile pas du tout (à éviter), ` ```should_panic ` attend une panique, ` ```text ` n'est pas du Rust. Une ligne qui commence par `# ` est compilée mais cachée dans la doc rendue : pratique pour un `# use ...;` ou un `# Ok::<(), Box<dyn std::error::Error>>(())` final qui permet d'utiliser `?` dans l'exemple.

**Venant de Python :** les docstrings deviennent des `///`, et les doctests sont activés d'office, sans `python -m doctest`. La convention de rustdoc pour une fonction : une première phrase courte (elle sert de résumé dans les listes), puis les détails, puis des sections `# Errors`, `# Panics` et `# Examples` si besoin. Clippy t'en rappelle certaines.

### 9.12 `should_panic`, tests qui renvoient `Result`, filtres

Trois formes de plus, toutes dans `tests/parse.rs` :

```rust
#[test]
fn avec_point_d_interrogation() -> Result<(), ErreurPorts> {
    let ports = parse("443,8443")?;
    println!("lu : {ports:?}");
    assert_eq!(ports.len(), 2);
    Ok(())
}

#[test]
#[should_panic(expected = "pas un port")]
fn unwrap_sur_une_erreur_panique() {
    parse("http").map_err(|e| e.to_string()).expect("pas un port");
}

#[test]
#[ignore = "lent : parcourt les 65535 ports"]
fn tous_les_ports() {
    assert_eq!(parse("1-65535").unwrap().len(), 65535);
}
```

- Un test qui renvoie `Result<(), E>` (avec `E: Debug`) échoue s'il rend `Err` : tu peux utiliser `?` au lieu d'un `unwrap()` par ligne. Le message d'échec affiche l'erreur avec `Debug`.
- `#[should_panic(expected = "...")]` réussit seulement si le test panique **avec un message qui contient ce texte**. Sans `expected`, n'importe quelle panique ferait l'affaire, y compris un bug sans rapport : mets toujours le texte.
- `#[ignore]` exclut le test des lancements ordinaires. `cargo test -- --ignored` ne lance que ceux-là, `cargo test -- --include-ignored` lance tout. Pour les tests lents, ou ceux qui ont besoin d'une base PostgreSQL (chapitre 14).

```
$ cargo test --test parse
test tous_les_ports ... ignored, lent : parcourt les 65535 ports
test erreurs ... ok
test avec_point_d_interrogation ... ok
test melange ... ok
test unwrap_sur_une_erreur_panique - should panic ... ok
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo test point
     Running tests/parse.rs (target/debug/deps/parse-d8c282f2433acb75)
test avec_point_d_interrogation ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
$ cargo test -q point -- --nocapture
lu : [443, 8443]
$ cargo test --test parse -- --ignored
test tous_les_ports ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

L'argument de `cargo test nom` est une **sous-chaîne** du nom complet du test (`ports::tests::un_port`), pas une expression régulière : `cargo test ports::` lance tout un module. Ce qui est avant `--` est pour Cargo (`--test parse` : un seul fichier de `tests/`, `--lib`, `--doc`, `--no-fail-fast` pour continuer après un binaire rouge), ce qui est après est pour le programme de test (`--nocapture`, `--ignored`, `--test-threads=1`, `--exact`). Par défaut la sortie d'un test qui passe est avalée ; `--nocapture` (ou `--show-output`) la montre.

### 9.13 `cargo doc`, `missing_docs`, et un mot sur les benchmarks

```
$ cargo doc --open
 Documenting portscan v0.1.0 (…/portscan)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.81s
   Generated …/portscan/target/doc/portscan/index.html
```

`cargo doc` construit la doc HTML de ta crate **et de toutes ses dépendances**, à la version exacte de ton `Cargo.lock`, et `--open` l'ouvre dans le navigateur. C'est docs.rs en local, hors ligne, et c'est la meilleure doc que tu auras sur tes dépendances (`--no-deps` pour seulement la tienne). `cargo doc --document-private-items` inclut les éléments privés, pour l'équipe.

`#![warn(missing_docs)]` en tête de `lib.rs` (le `!` dit que l'attribut s'applique à la crate entière) fait de chaque élément public non documenté un avertissement. Retire le `///` de `MAX` :

```
$ cargo build
warning: missing documentation for a constant
 --> src/ports.rs:8:1
  |
8 | pub const MAX: u16 = 65535;
  | ^^^^^^^^^^^^^^^^^^
  |
note: the lint level is defined here
 --> src/lib.rs:8:9
  |
8 | #![warn(missing_docs)]
  |         ^^^^^^^^^^^^
```

Mets-le dans toute bibliothèque : la doc fait partie de l'API, et ce lint coûte une ligne. Les labos 09, 10 et 12 l'activent.

**Les benchmarks.** `cargo bench` existe, mais le harnais intégré (`#[bench]`) est réservé à nightly. Sur stable, tout le monde utilise **criterion** : `cargo add --dev criterion`, un fichier `benches/parse.rs`, `[[bench]] name = "parse"` et `harness = false` dans `Cargo.toml`, puis `cargo bench`. Criterion répète la mesure, calcule un intervalle de confiance, et compare automatiquement avec le lancement précédent (« Performance has improved », « No change in performance detected »). Comme en Go, ce sont deux chiffres à comparer avant et après une optimisation, pas des chiffres absolus. Le labo 09 en fait une piste « pour aller plus loin ».

### 9.14 Pour le labo

Le [labo 09](../labs/09-ma-crate/README.md) te fait construire ta première vraie crate : une bibliothèque `semver_maison` en deux modules (`version` et `contrainte`), qui choisit son API avec `pub use`, des erreurs propres, `Ord` selon la précédence semver (`1.0.0-rc.1 < 1.0.0`), les contraintes exactement comme dans un `Cargo.toml` (`^1.2`, `~1.2.3`, `>=1.0, <2.0`), un binaire `semver` qui l'utilise comme un client, et les trois sortes de tests : unitaires dans chaque module, d'intégration dans `tests/`, et de doc sur `Version::parse` et `Contrainte::satisfait`. C'est le labo 09 du cours Go, en plus complet : tu y écris toi-même les règles que Cargo applique à chaque `cargo update`.

### À retenir

- Paquet = un `Cargo.toml` ; crate = une unité de compilation (une bibliothèque `lib.rs`, des binaires `main.rs` et `src/bin/*.rs`) ; module = un espace de noms dans une crate.
- Un fichier n'est compilé que s'il est déclaré par `mod nom;`, qui cherche `nom.rs` puis, pour ses sous-modules, `nom/sous.rs` ; `mod.rs` est l'ancienne forme.
- Tout est privé par défaut, visible dans le module et ses descendants ; `pub`, `pub(crate)` et `pub(super)` ouvrent ; `pub use` choisit l'API indépendamment des fichiers.
- `crate::` part de la racine, `super::` du parent, `self::` du module courant ; `use` ne crée qu'un raccourci, et il est obligatoire pour appeler une méthode de trait.
- Toujours `lib.rs` plus un `main.rs` minuscule : la logique se teste, les tests d'intégration ont quelque chose à importer.
- `"1.2"` dans `Cargo.toml` veut dire `^1.2` ; les features s'additionnent ; `cargo add`, `cargo update`, `cargo tree` ; commite `Cargo.lock`, toujours.
- Tests unitaires dans `#[cfg(test)] mod tests` avec `use super::*`, qui voient les privés ; tests d'intégration dans `tests/`, qui ne voient que l'API ; tests de doc dans les `///`, exécutés par `cargo test`.
- `#[should_panic(expected = ...)]`, les tests qui rendent `Result` pour utiliser `?`, `#[ignore]` et `-- --ignored`, `cargo test sous_chaine`, `-- --nocapture`.
- `cargo doc --open` documente ta crate et toutes tes dépendances ; `#![warn(missing_docs)]` dans toute bibliothèque ; criterion pour mesurer.

---

← [8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic](08-erreurs.md) · [Sommaire](../README.md) · [10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe](10-sous-le-capot.md) →
