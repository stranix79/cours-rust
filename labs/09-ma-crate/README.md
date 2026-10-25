# Labo 09 : Ma crate

**Chapitre** : [9. Modules, crates, Cargo, tests, documentation](../../cours/09-modules-crates-tests.md)

## Objectif
Construire ta première vraie crate : une bibliothèque `semver_maison` découpée en modules (`version`, `contrainte`), qui choisit son API avec `pub use`, et un binaire `semver` qui l'utilise comme le ferait n'importe quel client. Le sujet est celui du labo 09 du cours Go (versions sémantiques et contraintes `^` / `~`), en plus complet : les préversions (`1.0.0-rc.1 < 1.0.0`) et les contraintes exactement comme dans un `Cargo.toml`. À la fin tu sais où Rust range les fichiers, ce que `pub` ouvre, et tu as écrit les trois sortes de tests de Rust : unitaires (dans le module, avec accès aux privés), d'intégration (dans `tests/`, seulement l'API publique) et de documentation (les exemples des `///`, exécutés par `cargo test`).

## Consignes
1. Regarde l'arborescence : `Cargo.toml` (un bloc `[lib]` qui nomme la bibliothèque `semver_maison`, un `[[bin]]` qui nomme le binaire `semver`), `src/lib.rs` (la racine : `mod` et `pub use`, à lire, rien à écrire), `src/version.rs` et `src/contrainte.rs` (à compléter), `src/main.rs` (la CLI, fournie), `tests/semver.rs` (les tests d'intégration, fournis). Lance `cargo test` : ça compile, et ça échoue. `cargo test --no-fail-fast` continue après le premier binaire de test rouge et montre tout.
2. Lis les modules `tests` en bas de `version.rs` et `contrainte.rs` : ils appellent `lire_nombre`, `comparer_identifiants`, `lire_partielle`, qui sont privés. C'est permis parce que `mod tests` est un enfant du module. Essaie d'appeler `lire_nombre` depuis `tests/semver.rs` et lis l'erreur `E0425` ou `E0603` : c'est `pub` qui fait son travail.
3. `lire_nombre` (TODO 1) puis `Version::parse` (TODO 2) : le `v` et les espaces tolérés, le build après `+`, la préversion après le premier `-`, exactement trois nombres sans zéro en tête. Chaque faute a sa variante d'`ErreurVersion`. `cargo test version` puis `cargo test parse_versions`.
4. `comparer_identifiants` (TODO 3) et `Ord::cmp` (TODO 4) : la précédence semver. Le test `precedence_semver` prend la liste de la spécification (`1.0.0-alpha < 1.0.0-alpha.1 < … < 1.0.0-rc.1 < 1.0.0`). Note que `PartialEq` est écrit à la main et délègue à `cmp` : un `#[derive(PartialEq)]` comparerait aussi le build, et `==` dirait le contraire de `cmp`. `cargo test precedence`.
5. `Display` (TODO 5) : `1.2.3-beta.1+build.5`. Une fois fait, `to_string()` marche tout seul, et `cargo test affichage`.
6. `traduire` (TODO 6) et `Contrainte::satisfait` (TODO 7) : une contrainte est traduite une fois pour toutes en bornes simples, et `satisfait` ne fait que les tester. Les règles sont celles de Cargo : `^1.2` veut dire `>=1.2.0, <2.0.0`, `^0.2.3` veut dire `<0.3.0` (avant 1.0, le mineur casse), `~1.2.3` veut dire `<1.3.0`, une version nue vaut un `^`. Retire le `#![allow(dead_code)]` en tête de `contrainte.rs` quand c'est fait. `cargo test contrainte`.
7. `cargo test` : tout vert, y compris la ligne `Doc-tests semver_maison`. Change un exemple dans un `///` (mets `assert!(!caret.satisfait(&v("1.2.0")))`), relance : le test de doc casse. C'est une doc qui ne peut pas mentir.
8. `cargo doc --open` : ta doc, depuis tes commentaires, avec la page de la crate (le `//!` de `lib.rs`), les types et les exemples. Retire un `///` sur un champ public et recompile : `#![warn(missing_docs)]` te le reproche. Puis lance la CLI.
9. `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                        # tes tests (rouge tant que les TODO restent)
cargo test --no-fail-fast         # tous les binaires de test, même après un échec
cargo test version                # les tests dont le nom contient « version »
cargo test --doc                  # seulement les exemples de la doc
cargo doc --open                  # la doc HTML de ta crate
cargo run -- compare 1.2.3 1.10.0
cargo run -- satisfait "^1.2" 1.5.0 ; echo "code $?"
(cd solution && cargo test)       # la solution (vert)
(cd solution && cargo run -- sort 1.10.0 1.2.3 1.2.3-rc.1 0.9.0)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo test
     Running unittests src/lib.rs (…/target/debug/deps/semver_maison-97dbea0edc29ed6e)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/main.rs (…/target/debug/deps/semver-625f82f68f5a19d2)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/semver.rs (…/target/debug/deps/semver-9f5c04cb0384aeb8)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests semver_maison
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
$ cargo run -q -- compare 1.2.3 1.10.0
<
$ cargo run -q -- sort 1.10.0 1.2.3 1.2.3-rc.1 0.9.0 2.0.0-beta.11 2.0.0-beta.2
0.9.0
1.2.3-rc.1
1.2.3
1.10.0
2.0.0-beta.2
2.0.0-beta.11
$ cargo run -q -- satisfait "^1.2" 1.5.0 ; echo "code $?"
oui
code 0
$ cargo run -q -- satisfait "^1.2" 2.0.0 ; echo "code $?"
non
code 1
$ cargo run -q -- compare 1.2 1.0.0 ; echo "code $?"
semver : "1.2" : attendu MAJEUR.MINEUR.CORRECTIF
code 2
```
(Sorties de `cargo test` raccourcies aux lignes `Running` et `test result`.) Quatre binaires de test pour un paquet : les tests unitaires de la bibliothèque, ceux du binaire, chaque fichier de `tests/`, et les tests de doc.

## Pour aller plus loin
- Ajoute un benchmark : `cargo add --dev criterion`, un fichier `benches/parse.rs` avec `criterion_group!` et `criterion_main!`, et `[[bench]] name = "parse" harness = false` dans `Cargo.toml`. `cargo bench` mesure `Version::parse` en nanosecondes, avec un intervalle de confiance et la comparaison avec la mesure précédente.
- Ajoute une commande `semver bump minor 1.2.3` qui rend `1.3.0` (et retire préversion et build), avec ses tests dans le module `tests` de `main.rs` d'abord. Puis compare ta crate avec la vraie, `semver` sur crates.io (celle que Cargo utilise), en lisant sa page sur docs.rs : mêmes noms ou presque, `VersionReq` pour la contrainte.
- Transforme le paquet en deux paquets d'un espace de travail : la bibliothèque d'un côté, la CLI de l'autre qui en dépend par `semver_maison = { path = "../semver-maison" }`. C'est la structure de ripgrep (une dizaine de crates `grep-*` et un binaire `rg`). Le chapitre 10 passe ensuite sous le capot de ce que tu viens d'écrire : où vivent ces `String` et ces `Vec`, et ce que coûte un `Box`.
