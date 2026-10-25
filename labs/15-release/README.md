# Labo 15 : hostinfo, un outil prêt à livrer

**Chapitre** : [15. Qualité, build, release, Docker](../../cours/15-qualite-build-release.md)

## Objectif
Amener un tout petit outil au niveau « livrable ». `hostinfo` affiche la machine, le système, l'architecture, le nombre de cœurs, sa propre version et depuis combien de temps la machine tourne, en texte ou en JSON. Le code tient en cent lignes ; le sujet du labo est tout ce qu'il y a autour : des lints versionnés dans `Cargo.toml`, un `rustfmt.toml`, un profil release qui divise la taille du binaire par six, un `Makefile`, une compilation croisée vers Linux depuis ton Mac, une image Docker multi-étapes de moins d'un méga-octet, un workflow GitHub Actions sur trois systèmes, et la version de `Cargo.toml` qui arrive jusque dans `--version`. C'est la transposition du labo 15 du cours Go, et le squelette de tous tes futurs outils d'infra en Rust.

## Les fichiers, un par un
| Fichier | Rôle |
|---|---|
| `Cargo.toml` | le paquet, ses dépendances (`clap`, `serde`, `serde_json`), et les sections `[lints.rust]` et `[lints.clippy]` : `unsafe_code` interdit, le groupe `pedantic` en avertissement, `unwrap_used` en avertissement, deux règles pédantes désactivées en connaissance de cause. Ces règles s'appliquent à `cargo clippy` chez toi, dans ton éditeur et en CI, sans option à répéter. |
| `rustfmt.toml` | les réglages de `cargo fmt` : l'édition, une largeur de 100 colonnes, fins de ligne Unix. Peu d'options, exprès. |
| `.cargo/config.toml` | le `[profile.release]` : `lto = "fat"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`. Il est ici et pas dans `Cargo.toml` parce que le labo est membre de l'espace de travail du cours, où Cargo ignore les profils des membres (il le dit par un avertissement). Dans ton projet, ce bloc va dans `Cargo.toml`. |
| `Makefile` | les commandes à retenir : `build`, `run`, `test`, `lint`, `release`, `cross`, `docker`, `clean`. |
| `Dockerfile` | deux étapes : `rust:1.93-alpine` compile un binaire musl statique, `scratch` ne contient que lui. Les dépendances sont compilées dans une couche à part, mise en cache tant que `Cargo.toml` ne change pas. |
| `.dockerignore` | ce que `docker build` n'envoie pas : `target/`, `solution/`, `.git/`. |
| `.github/workflows/ci.yml` | `fmt` et `clippy` sur Linux, puis `test`, `build --release` et `--version` sur Linux, macOS et Windows, avec le cache `Swatinem/rust-cache`, et un binaire musl en artefact sur un tag `v*`. Exemple à copier à la racine de ton dépôt : GitHub ne lit pas les workflows d'un sous-dossier. |
| `solution/verifier.sh` | lance les tests, le build release, vérifie la présence des fichiers de livraison et que `--version` affiche la version de `Cargo.toml`. |

## Consignes
1. Lis `Cargo.toml`, `rustfmt.toml` et `.cargo/config.toml`, puis `src/lib.rs` et `tests/hostinfo.rs`. Lance `cargo test` : ça compile, six tests sur sept échouent (`not yet implemented: TODO 1`).
2. **TODO 1**, `lire_proc_uptime(contenu) -> Option<u64>` : le premier nombre de `/proc/uptime` (`"4591920.53 36581233.10\n"`), sans sa partie décimale. Chaîne vide, texte ou nombre négatif : `None`. `cargo test proc_uptime`.
3. **TODO 2**, `lire_kern_boottime(sortie, maintenant) -> Option<u64>` : la sortie de `sysctl -n kern.boottime` sous macOS (`"{ sec = 1788357618, usec = 302303 } ..."`), et `maintenant - sec`. `checked_sub` pour qu'un démarrage dans le futur donne `None` et pas un dépassement. `cargo test kern_boottime`.
4. **TODO 3**, `format_duree(secondes) -> String` : `42 s`, `5 min`, `3 h 12 min`, `53 j 3 h 32 min`. `cargo test durees`.
5. **TODO 4**, `format_texte(&Infos) -> String` : six lignes, libellés alignés sur 13 caractères (`{libelle:<13}` compte des caractères, pas des octets : `système` et `cœurs` s'alignent). `cargo test texte`.
6. **TODO 5**, la version : remplace `"0.0.0"` par `env!("CARGO_PKG_VERSION")`. Change `version` dans `Cargo.toml` en `1.3.0`, relance `cargo test version` : le test échoue, il attend `1.2.0`. C'est le comportement voulu : la version n'est écrite qu'à un seul endroit. Remets `1.2.0`.
7. **TODO 6 et 7**, dans `src/main.rs` : `version` dans `#[command(...)]` (`cargo run -- --version` doit afficher `hostinfo 1.2.0`), puis la sortie `--json` avec `serde_json::to_string_pretty`. Essaie d'abord avec `.unwrap()` et lance `cargo clippy` : `[lints]` le signale (``used `unwrap()` on a `Result` value``, et `-D warnings` en fait une erreur en CI). Fais un `match`.
8. Retire la ligne `#![allow(...)]` en haut de `lib.rs`, puis `make lint` : `cargo fmt --check` et `cargo clippy --all-targets -- -D warnings` ne disent rien. Écris volontairement `pub fn f(x: &String) -> usize { x.len() }` dans `lib.rs` et relance : deux remarques. ``writing `&String` instead of `&str` `` vient des règles par défaut de clippy ; ``this function could have a `#[must_use]` attribute`` vient de pedantic. Retire la fonction.
9. `make release` : compare la taille avec celle de `cargo build` (debug, dans `../../target/debug/hostinfo`). Puis commente une à une les lignes de `.cargo/config.toml` et relance `make release` à chaque fois : le tableau du chapitre 15 donne les chiffres sur un Mac M1.
10. `make cross CIBLES="x86_64-unknown-linux-musl aarch64-unknown-linux-musl"` : deux binaires Linux statiques construits depuis ton Mac, liés par `rust-lld` (livré avec Rust). Vérifie avec `file`. `x86_64-pc-windows-gnu` échoue sans MinGW (`error calling dlltool 'x86_64-w64-mingw32-dlltool'`) : `brew install mingw-w64`, ou `cargo zigbuild`, ou `cross`, voir le chapitre.
11. Si tu as Docker (ou Podman) : `make docker`, puis `docker run --rm hostinfo:1.2.0 --json`. L'image fait moins d'un méga-octet et n'a pas de shell (`docker run --rm --entrypoint sh hostinfo:1.2.0` échoue). Sans Docker, lis le `Dockerfile` : chaque ligne est commentée.
12. `(cd solution && bash verifier.sh)` : tout est vert.

## Comment lancer
```bash
cargo test                          # tes tests (rouge tant que les TODO restent)
cargo test durees                   # un seul test
cargo run                           # ton programme, en texte
cargo run -- --json                 # en JSON
cargo run -- --version
make lint                           # cargo fmt --check, puis clippy -D warnings
make release                        # le binaire optimisé et sa taille
make cross CIBLES="x86_64-unknown-linux-musl aarch64-unknown-linux-musl"
make docker                         # si Docker est installé
(cd solution && cargo test)         # la solution (vert)
(cd solution && bash verifier.sh)   # tests, release, fichiers, version
```

## Sortie attendue
Les valeurs dépendent de ta machine, la forme non :
```
$ cargo run -q
machine      : morningstar
système      : macos
architecture : aarch64
cœurs        : 10
version      : 1.2.0
uptime       : 53 j 4 h 4 min
$ cargo run -q -- --json
{
  "machine": "morningstar",
  "systeme": "macos",
  "architecture": "aarch64",
  "coeurs": 10,
  "version": "1.2.0",
  "uptime_secondes": 4593866
}
$ cargo run -q -- --version
hostinfo 1.2.0

$ make release
cargo build --release
    Finished `release` profile [optimized] target(s) in 10.44s
552784 octets : /Users/stranix/git/stranix/cours-rust/target/release/hostinfo

$ make cross CIBLES="x86_64-unknown-linux-musl aarch64-unknown-linux-musl"
  x86_64-unknown-linux-musl
    Finished `release` profile [optimized] target(s) in 9.81s
  aarch64-unknown-linux-musl
    Finished `release` profile [optimized] target(s) in 9.33s
$ file ../../target/aarch64-unknown-linux-musl/release/hostinfo
../../target/aarch64-unknown-linux-musl/release/hostinfo: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, stripped

$ make docker
docker build -t hostinfo:1.2.0 .
docker images hostinfo
REPOSITORY          TAG      IMAGE ID       CREATED          SIZE
localhost/hostinfo  1.2.0    ca2010ee1bb6   11 seconds ago   663 kB
$ docker run --rm hostinfo:1.2.0
machine      : 9a3b339506f5
système      : linux
architecture : aarch64
cœurs        : 2
version      : 1.2.0
uptime       : 17 h 24 min
```

Dans le conteneur, la machine est l'identifiant du conteneur, l'uptime celui de la machine virtuelle de Docker ou Podman, et le nom de la machine vient de `/proc/sys/kernel/hostname` : il n'y a pas de commande `hostname` dans `scratch`.

## Pour aller plus loin
- Ajoute `--format <texte|json|env>` avec un `enum` qui dérive `clap::ValueEnum` à la place du booléen `--json`, et une sortie `HOSTINFO_MACHINE=...` qu'un script shell peut `eval`. Un test par format.
- Ajoute la journalisation avec `tracing` et `tracing-subscriber` (`RUST_LOG=debug hostinfo` doit dire d'où vient chaque valeur : `/proc`, `sysctl` ou défaut), en écrivant sur stderr pour que `--json` reste lisible par `jq`.
- Installe cargo-dist (`cargo install cargo-dist`), lance `dist init` dans une copie du labo hors de l'espace de travail du cours, et lis le workflow `release.yml` qu'il génère : archives, sommes de contrôle, installateur shell, pour chaque cible. Le chapitre 16 assemble ensuite tout le cours dans un service.
