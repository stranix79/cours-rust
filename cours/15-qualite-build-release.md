# 15. Qualité, build, release, Docker

*Rust de zéro à la prod : chapitre 15 sur 16.* ← [14. Bases de données : SQLite et PostgreSQL](14-bases-de-donnees.md) · [Sommaire](../README.md) · [16. Projet final et pour aller plus loin](16-projet-final-et-suite.md) →

Ton programme marche sur ta machine, lancé avec `cargo run`. Ce chapitre le fait sortir de là : vérifié par des outils, testé en CI sur trois systèmes, compilé avec un profil qui divise sa taille par six, construit pour Linux depuis ton Mac, emballé dans une image Docker de moins d'un méga-octet, configuré par l'environnement, et qui dit ce qu'il fait dans un journal qu'on peut lire. Rien de tout ça n'est propre à Rust : tu connais `make`, Docker, GitHub Actions. Ce qui change, ce sont les réglages, et deux ou trois endroits où Rust demande un geste que Go faisait tout seul.

Tout ce qui suit est appliqué dans le [labo 15](../labs/15-release/README.md), sur un petit outil, `hostinfo`, qui sert de cobaye.

### 15.1 clippy : ce que le compilateur laisse passer

Le compilateur Rust refuse déjà beaucoup de choses. clippy, livré avec rustup, ajoute quelques centaines de règles sur ce qui compile mais s'écrit mieux autrement. Un programme volontairement maladroit :

```rust
use std::collections::HashMap;

fn moyenne(valeurs: &Vec<f64>) -> f64 {
    let mut total = 0.0;
    for i in 0..valeurs.len() {
        total = total + valeurs[i];
    }
    total / valeurs.len() as f64
}

fn port(config: &HashMap<String, String>) -> u16 {
    config.get("port").unwrap().parse().unwrap()
}

fn main() {
    let latences = vec![12.0, 15.5, 9.25];
    println!("moyenne : {}", moyenne(&latences));
    let mut config = HashMap::new();
    config.insert("port".to_string(), "5432".to_string());
    if config.len() == 0 {
        return;
    }
    println!("port : {}", port(&config));
}
```

```
$ cargo clippy
warning: writing `&Vec` instead of `&[_]` involves a new object where a slice will do
 --> src/main.rs:3:21
  |
3 | fn moyenne(valeurs: &Vec<f64>) -> f64 {
  |                     ^^^^^^^^^ help: change this to: `&[f64]`
  |
  = note: `#[warn(clippy::ptr_arg)]` on by default

warning: the loop variable `i` is only used to index `valeurs`
 --> src/main.rs:5:14
  |
5 |     for i in 0..valeurs.len() {
  |              ^^^^^^^^^^^^^^^^
  |
help: consider using an iterator
  |
5 -     for i in 0..valeurs.len() {
5 +     for <item> in &valeurs {
  |

warning: manual implementation of an assign operation
 --> src/main.rs:6:9
  |
6 |         total = total + valeurs[i];
  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace it with: `total += valeurs[i]`

warning: length comparison to zero
  --> src/main.rs:20:8
   |
20 |     if config.len() == 0 {
   |        ^^^^^^^^^^^^^^^^^ help: using `is_empty` is clearer and more explicit: `config.is_empty()`
```

Quatre remarques, chacune avec la correction. C'est le niveau par défaut, et il se trompe rarement : un projet Rust sérieux passe `cargo clippy` sans un avertissement. Les règles sont rangées en groupes :

| Groupe | Contenu | Réglage |
|---|---|---|
| `clippy::all` (défaut) | correctness, suspicious, style, complexity, perf | toujours |
| `clippy::pedantic` | remarques exigeantes, parfois discutables | à bon escient |
| `clippy::nursery` | règles en cours de mise au point | non |
| `clippy::restriction` | interdictions à la carte (`unwrap_used`, `print_stdout`...) | une par une, jamais le groupe |
| `clippy::cargo` | les métadonnées du `Cargo.toml` | pour une crate publiée |

Avec `pedantic` et `unwrap_used` en plus, trois remarques de plus sur le même programme :

```
$ cargo clippy -- -W clippy::pedantic -W clippy::unwrap_used
warning: casting `usize` to `f64` causes a loss of precision on targets with 64-bit wide pointers (`usize` is 64 bits wide, but `f64`'s mantissa is only 52 bits wide)
 --> src/main.rs:8:13
warning: used `unwrap()` on a `Result` value
  --> src/main.rs:12:5
warning: used `unwrap()` on an `Option` value
  --> src/main.rs:12:5
```

Les deux `unwrap` sont de vraies bombes : un fichier de configuration sans `port`, et le service panique au démarrage. La conversion `usize` vers `f64`, elle, est sans danger ici (personne n'a 2^52 latences). C'est tout `pedantic` : de bonnes remarques, et quelques-unes qu'il faut savoir refuser. Mon réglage : `pedantic` en avertissement pour un outil ou un service que tu maintiens, désactivé règle par règle quand il a tort, et jamais sur un prototype.

Les règles se versionnent avec le code, dans la section `[lints]` du `Cargo.toml`. Celle du labo 15 :

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
# pedantic : le groupe des remarques de style exigeantes. priority = -1 pour
# que les réglages individuels ci-dessous l'emportent sur le groupe.
pedantic = { level = "warn", priority = -1 }
# On interdit unwrap() dans le code (expect() avec un message reste permis).
unwrap_used = "warn"
# Deux règles pedantic qu'on désactive en connaissance de cause.
module_name_repetitions = "allow"
missing_panics_doc = "allow"
```

Ton éditeur, `cargo clippy` chez toi et la CI lisent les mêmes règles, sans option à répéter. Les niveaux sont `allow`, `warn`, `deny` (erreur, qu'un `#[allow]` local peut lever) et `forbid` (erreur, que rien ne lève). En CI, on lance `cargo clippy -- -D warnings` : tout avertissement devient une erreur, sans rendre le développement local pénible.

Et quand une règle a tort à un endroit précis, on le dit à cet endroit, avec la raison. `#[expect(...)]` vaut mieux que `#[allow(...)]` : il fait taire la règle **et** prévient le jour où elle ne se déclenche plus.

```rust
#![warn(clippy::pedantic)]

/// La moyenne des latences, en millisecondes.
#[expect(
    clippy::cast_precision_loss,
    reason = "moins de 2^52 mesures : la conversion est exacte"
)]
fn moyenne(valeurs: &[f64]) -> f64 {
    valeurs.iter().sum::<f64>() / valeurs.len() as f64
}

/// Une version qui n'a plus besoin de l'exception.
#[expect(clippy::cast_precision_loss, reason = "copié-collé, plus nécessaire")]
fn somme(valeurs: &[f64]) -> f64 {
    valeurs.iter().sum()
}

fn main() {
    println!("{} {}", moyenne(&[12.0, 15.5, 9.25]), somme(&[1.0]));
}
```

```
$ cargo clippy
warning: this lint expectation is unfulfilled
  --> src/bin/expect.rs:13:10
   |
13 | #[expect(clippy::cast_precision_loss, reason = "copié-collé, plus nécessaire")]
   |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: copié-collé, plus nécessaire
```

Les exceptions mortes ne s'accumulent plus.

**Venant de Go :** `go vet` est le niveau par défaut de clippy, et golangci-lint avec sa configuration `.golangci.yml` est clippy avec `[lints]`. Une différence de culture : en Go, golangci-lint est une dépendance à installer et à épingler ; clippy est livré avec le compilateur, dans la même version, et tout le monde l'a.

**Venant de Python :** c'est `ruff check`, avec ses familles de règles (`E`, `F`, `B`, `PL`) qui ressemblent aux groupes de clippy, et la configuration dans `pyproject.toml` comme ici dans `Cargo.toml`. Ruff est d'ailleurs écrit en Rust, et sa rapidité vient de là.

### 15.2 rustfmt.toml

`cargo fmt` formate tout le projet ; `cargo fmt --check` dit seulement s'il y a quelque chose à formater, et sort en erreur le cas échéant : c'est la version CI. Le style par défaut est celui de toute la communauté, et c'est sa principale qualité. Un `rustfmt.toml` à la racine peut en ajuster quelques points :

```toml
edition = "2024"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
newline_style = "Unix"
```

Peu d'options sont stables, exprès : comme `gofmt`, rustfmt doit donner le même résultat chez tout le monde. Mon conseil : ne touche à rien, sauf peut-être `max_width`. Une équipe qui débat de la largeur de ses lignes a trouvé une façon coûteuse de ne pas travailler.

### 15.3 `cargo doc`, et les exemples qui sont des tests

Les commentaires `///` (sur l'élément qui suit) et `//!` (sur le module qui les contient) sont du Markdown, et `cargo doc` en fait un site HTML, avec la documentation de toutes tes dépendances et des liens entre elles. `cargo doc --no-deps --open` pour ta crate seule. Et le bloc de code d'un commentaire de documentation est compilé et exécuté par `cargo test` :

```rust
//! ch15 : un exemple de documentation.

/// Convertit des millisecondes en secondes entières (arrondi vers le bas).
///
/// ```
/// assert_eq!(ch15::en_secondes(1_500), 1);
/// assert_eq!(ch15::en_secondes(999), 0);
/// ```
#[must_use]
pub fn en_secondes(ms: u64) -> u64 {
    ms / 1000
}
```

```
$ cargo test --doc
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Un exemple de documentation qui ne compile plus casse les tests : la doc ne ment jamais longtemps. C'est le chapitre 9 en action, et c'est ce qui rend la documentation de docs.rs aussi fiable.

### 15.4 `cargo audit` et `cargo deny`

Deux outils à installer (`cargo install cargo-audit cargo-deny`), et à mettre en CI dès qu'un service part en production.

- **`cargo audit`** compare ton `Cargo.lock` à la base RustSec des vulnérabilités connues et sort en erreur si une de tes dépendances, même indirecte, est touchée. C'est le `govulncheck` de Go, en plus simple : il regarde les versions, pas le code appelé.
- **`cargo deny`** va plus loin avec un fichier `deny.toml` : licences autorisées (pas de GPL dans un binaire que tu distribues ?), crates interdites (OpenSSL, par exemple), doublons de versions, sources autorisées (crates.io seulement, pas un dépôt git inconnu), et les mêmes avis RustSec.

Un `cargo audit` vert ne veut pas dire « sûr ». Il veut dire qu'aucune faille *publiée* ne touche tes versions. C'est le minimum, et il coûte trente secondes de CI.

### 15.5 La CI : GitHub Actions

Le workflow du labo 15, l'essentiel :

```yaml
env:
  CARGO_TERM_COLOR: always
  # Un avertissement du compilateur casse la CI, pas seulement ceux de clippy.
  RUSTFLAGS: -D warnings

jobs:
  # Le formatage et clippy ne dépendent pas du système : une seule machine.
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2   # met target/ et ~/.cargo en cache entre deux runs
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings

  # Les tests et le build sur les trois systèmes.
  test:
    needs: lint
    strategy:
      fail-fast: false              # voir les trois résultats, même si l'un échoue
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test
      - run: cargo build --release
      - run: cargo run --release -- --version
```

Trois choix à comprendre. `dtolnay/rust-toolchain` installe la chaîne demandée (`stable`, ou une version précise comme `1.93`) ; un `rust-toolchain.toml` à la racine du dépôt fixe la version pour tout le monde, CI comprise. `Swatinem/rust-cache` est indispensable : sans lui, chaque run recompile tokio et serde depuis zéro, et une CI de quarante secondes en prend six minutes. Et la matrice sur trois systèmes coûte peu et attrape ce qu'on ne voit jamais sur son Mac : un chemin écrit avec `/` en dur, un `/proc` qui n'existe que sous Linux, un fichier ouvert qui empêche sa suppression sous Windows. Le fichier complet du labo ajoute un job qui, sur un tag `v*`, construit le binaire Linux statique et le joint comme artefact.

### 15.6 Le profil release : ce que chaque réglage change

`cargo build` compile en mode *debug* : vite, avec les vérifications de débordement et les infos de débogage. `cargo build --release` optimise. Entre les deux, le profil `[profile.release]` se règle, et les réglages par défaut sont faits pour compiler vite, pas pour livrer petit. Ceux du labo 15, mesurés un par un sur `hostinfo` (clap, serde, serde_json), Rust 1.93, Mac M1, durée d'un build complet dépendances comprises :

| Profil | Taille | Build |
|---|---|---|
| `cargo build` (debug) | 3 298 056 octets | |
| `--release`, profil par défaut | 936 608 octets | 8 s |
| + `strip = true` | 778 464 octets | 8 s |
| + `lto = "fat"` | 652 288 octets | 15 s |
| + `codegen-units = 1` | 619 088 octets | 10 s |
| + `panic = "abort"` | 552 768 octets | 11 s |
| + `opt-level = "z"` | 454 512 octets | 9 s |

Ce que fait chacun, et ce qu'il coûte :

- **`strip = true`** retire la table des symboles du binaire. Par défaut, depuis Rust 1.77, la release retire déjà les infos de débogage (`strip = "debuginfo"`) ; `true` enlève aussi les noms de fonctions. Le coût : une trace de panique sans noms de fonctions, et un `perf` ou un `gdb` aveugles. Pour un binaire qu'on distribue, c'est le bon réglage.
- **`lto = "fat"`** (*link-time optimization*) optimise le programme entier à l'édition de liens, à travers toutes les crates : du code mort de clap disparaît, des fonctions de serde sont mises en ligne dans les tiennes. Plus petit, souvent un peu plus rapide, et une édition de liens nettement plus longue. `lto = "thin"` est un compromis.
- **`codegen-units = 1`** compile chaque crate en une seule unité au lieu de seize en parallèle : moins de parallélisme à la compilation, plus d'optimisation entre fonctions.
- **`panic = "abort"`** : un panic arrête le processus tout de suite, sans dérouler la pile ni appeler les destructeurs. Le code de déroulement disparaît du binaire. Le prix : `catch_unwind` ne marche plus, et une tâche tokio qui panique tue tout le service au lieu de rendre une `Err` à son `JoinHandle`. Pour un outil en ligne de commande, aucun problème ; pour un serveur, réfléchis-y (le labo 16 ne le met pas).
- **`opt-level = "z"`** optimise pour la taille plutôt que pour la vitesse. Utile pour l'embarqué ou WebAssembly ; pour un service, garde `3`, le défaut.

Le reste du profil par défaut est bon. Et un piège propre au cours : dans un espace de travail Cargo, seuls les profils du `Cargo.toml` racine comptent. Le labo 15 met donc le sien dans `.cargo/config.toml`, qui accepte la même section ; dans ton projet à toi, il va dans `Cargo.toml`.

### 15.7 Ce qui pèse dans le binaire : `cargo bloat`

`cargo install cargo-bloat`, puis :

```
$ cargo bloat --release --crates -n 8
 File  .text     Size Crate
37.7%  60.2% 253.9KiB std
22.3%  35.7% 150.4KiB clap_builder
 2.0%   3.2%  13.6KiB hostinfo
 0.6%   0.9%   3.8KiB [Unknown]
 0.4%   0.7%   2.9KiB anstream
 0.3%   0.4%   1.9KiB strsim
 0.3%   0.4%   1.7KiB serde_json
 0.2%   0.4%   1.6KiB anstyle
62.6% 100.0% 421.8KiB .text section size, the file size is 674.1KiB
```

Notre code pèse 13,6 Kio. La bibliothèque standard en pèse 254, clap 150. Si tu devais gagner encore 150 Kio, c'est là : une crate d'arguments plus légère (`lexopt`, `argh`), ou `std::env::args` à la main pour un outil à deux options. `cargo bloat -n 20` sans `--crates` liste les plus grosses fonctions. Pour la plupart des services, la question ne se pose pas : un binaire de cinq méga-octets dans une image de cinq méga-octets, c'est déjà vingt fois moins qu'une image Python.

### 15.8 Compilation croisée : musl, zigbuild, cross

`rustup target add` installe la bibliothèque standard précompilée d'une autre cible ; `cargo build --target ...` compile pour elle. Les cibles qu'un sysadmin utilise :

| Cible | Pour |
|---|---|
| `x86_64-unknown-linux-gnu` | Linux x86_64 classique, lié à la glibc du système |
| `x86_64-unknown-linux-musl` | Linux x86_64, binaire **statique**, tourne partout, image `scratch` |
| `aarch64-unknown-linux-musl` | Raspberry Pi 4 et 5, serveurs ARM (Graviton, Ampere), statique |
| `aarch64-apple-darwin` | Mac Apple Silicon |
| `x86_64-pc-windows-gnu` | Windows, avec la chaîne MinGW |

Le compilateur Rust sait produire du code pour toutes ces cibles depuis n'importe quelle machine. Ce qui manque, c'est l'**éditeur de liens** de la cible. Depuis un Mac :

```
$ rustup target add x86_64-unknown-linux-musl
$ cargo build --release --target x86_64-unknown-linux-musl
error: linking with `cc` failed: exit status: 1
  = note: ld: unknown options: --as-needed -Bstatic -Bdynamic --eh-frame-hdr -z --gc-sections -z -z --strip-debug
          clang: error: linker command failed with exit code 1 (use -v to see invocation)
```

Le `ld` d'Apple ne sait pas lier un binaire ELF. Trois solutions, de la plus légère à la plus lourde :

1. **`rust-lld`**, l'éditeur de liens de LLVM livré avec rustup. Pour les cibles musl, Rust embarque tout ce qu'il faut (le `crt1.o`, la libc musl) et `rust-lld` suffit, à condition que ton code et tes dépendances soient du Rust pur :
   ```
   $ CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld cargo build --release --target x86_64-unknown-linux-musl
   $ file target/x86_64-unknown-linux-musl/release/hostinfo
   target/x86_64-unknown-linux-musl/release/hostinfo: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked, stripped
   ```
   C'est ce que fait la cible `cross` du `Makefile` du labo, pour les deux cibles musl.
2. **`cargo zigbuild`** (`cargo install cargo-zigbuild`, plus Zig : `brew install zig` ou `pip install ziglang`). Zig sert de compilateur C et d'éditeur de liens pour n'importe quelle cible. Indispensable dès qu'une dépendance compile du C : rusqlite avec `bundled`, `ring` pour la cryptographie. `cargo zigbuild --release --target aarch64-unknown-linux-musl`, et c'est tout.
3. **`cross`** (`cargo install cross`) lance la compilation dans un conteneur Docker qui a la chaîne complète de la cible. Le plus robuste pour les cas difficiles, et il faut Docker.

Pour Windows depuis un Mac, `x86_64-pc-windows-gnu` réclame MinGW (`error calling dlltool 'x86_64-w64-mingw32-dlltool'` sans lui) : `brew install mingw-w64`, ou zigbuild, ou la CI sur `windows-latest`. En pratique, c'est la CI qui construit les binaires pour les systèmes que tu n'as pas.

**Venant de Go :** c'est le point où Go est imbattable. `GOOS=linux GOARCH=arm64 CGO_ENABLED=0 go build` produit un binaire statique depuis n'importe où, sans rien installer, parce que Go a son propre éditeur de liens et n'utilise pas la libc. Rust lie par défaut contre la libc du système (glibc sous Linux) : le binaire statique, il faut le demander avec la cible musl, et fournir un éditeur de liens. Une fois `rust-lld` ou zigbuild en place, c'est une commande, mais c'est une commande de plus que Go.

### 15.9 `cargo install`, et cargo-dist

`cargo install --path .` compile ta crate en release et copie le binaire dans `~/.cargo/bin`, qui est dans ton `PATH` depuis l'installation de rustup. C'est le moyen le plus court d'avoir ton outil partout sur ta machine ; `cargo install ripgrep` fait la même chose depuis crates.io. Avec `--root dossier`, il installe ailleurs :

```
$ cargo install --path . --root ../installe
     Locking 28 packages to latest Rust 1.85 compatible versions
    Finished `release` profile [optimized] target(s) in 12.00s
  Installing .../installe/bin/hostinfo
   Installed package `hostinfo v1.2.0 (.../taille)` (executable `hostinfo`)
```

Remarque la première ligne : « latest Rust 1.85 compatible versions ». Le `Cargo.toml` du labo déclare `rust-version = "1.85"`, et avec le résolveur de l'édition 2024, Cargo choisit des versions de dépendances qui compilent avec cette version de Rust. Déclarer `rust-version` est le meilleur moyen d'éviter la surprise d'une dépendance qui exige un compilateur plus récent que celui de ta CI (sqlx 0.9 et Rust 1.94 au chapitre 14).

Pour distribuer à d'autres que toi, [cargo-dist](https://opensource.axo.dev/cargo-dist/) fait ce que goreleaser fait pour Go. `dist init` pose sa configuration dans le dépôt et génère un workflow GitHub Actions qui, sur un `git tag v1.2.0 && git push --tags`, construit les binaires de chaque cible, les archives, les sommes de contrôle, un installateur `curl | sh` et une formule Homebrew, et crée la page Releases.

**Venant de Go :** goreleaser lit son `.goreleaser.yaml` et fait tout depuis un seul runner, grâce à la compilation croisée gratuite de Go. cargo-dist génère un workflow qui construit chaque cible sur son système natif (Linux, macOS, Windows) : plus de runners, moins de magie, et le même résultat sur la page Releases.

### 15.10 La version dans le binaire

Go injecte la version à l'édition de liens avec `-ldflags "-X main.version=1.4.0"`. Rust la lit dans `Cargo.toml`, à la compilation. Cargo définit `CARGO_PKG_VERSION` (et `CARGO_PKG_NAME`, `CARGO_PKG_AUTHORS`...) pour chaque compilation, et la macro `env!` la lit :

```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

`env!` échoue à la compilation si la variable manque : impossible de livrer un binaire sans version. Et clap la lit tout seul : `#[command(version)]` sur la struct des options ajoute `--version` et `-V`.

```
$ hostinfo --version
hostinfo 1.2.0
```

La version n'est écrite qu'à un endroit, `Cargo.toml`, et c'est le tag git qui doit la suivre, pas l'inverse. Si tu veux aussi le commit dans le binaire, un fichier `build.rs` peut lancer `git rev-parse --short HEAD` et le passer par `cargo:rustc-env=COMMIT=...`, que `env!("COMMIT")` lit ensuite.

### 15.11 Docker multi-étapes : moins d'un méga-octet

Le `Dockerfile` du labo 15 :

```dockerfile
FROM rust:1.93-alpine AS build
WORKDIR /src

# Les dépendances d'abord, seules, avec des sources vides : cette couche
# (la plus longue, clap et serde à compiler) n'est reconstruite que si
# Cargo.toml change. C'est le principe de cargo-chef, fait à la main.
COPY Cargo.toml ./
COPY .cargo .cargo
RUN mkdir src \
 && echo 'fn main() {}' > src/main.rs \
 && touch src/lib.rs \
 && cargo build --release \
 && rm -rf src

# Puis le vrai code. touch : pour que Cargo voie des sources plus récentes
# que la compilation à vide de l'étape précédente.
COPY src src
RUN touch src/main.rs src/lib.rs \
 && cargo build --release \
 && cp target/release/hostinfo /hostinfo

FROM scratch
COPY --from=build /hostinfo /hostinfo
USER 65534:65534
ENTRYPOINT ["/hostinfo"]
```

L'image `rust:1.93-alpine` compile pour musl par défaut : le binaire est statique, sans dépendance à une libc, et tourne sur `scratch`, l'image vide. La première étape pèse un giga-octet et disparaît. Le résultat, construit ici avec Podman (Docker donne la même chose) :

```
$ docker images
REPOSITORY            TAG          IMAGE ID      CREATED         SIZE
localhost/hostinfo    1.2.0        ca2010ee1bb6  11 seconds ago  663 kB
docker.io/library/rust  1.93-alpine  7c55fd8327d8  8 months ago    1.04 GB
$ docker run --rm hostinfo:1.2.0 --json
{
  "machine": "cf4d7956fd45",
  "systeme": "linux",
  "architecture": "aarch64",
  "coeurs": 2,
  "version": "1.2.0",
  "uptime_secondes": 62668
}
$ podman run --rm --entrypoint sh hostinfo:1.2.0
Error: preparing container 93c71db5b09b for attach: crun: executable file `sh` not found in $PATH: No such file or directory: OCI runtime attempted to invoke a command that was not found
```

663 Ko, et pas de shell : un attaquant qui entre n'a rien à exécuter. Le service `sondes` du chapitre 16, avec tokio, axum et SQLite compilé dedans, fait une image de 5,48 Mo.

Trois points d'attention :

- **Le cache des dépendances.** Sans l'étape à sources vides, chaque modification d'une ligne de ton code recompile tokio et serde dans le `docker build`. Le truc fait à la main (un `main.rs` vide, un build, puis le vrai code) marche bien pour un paquet simple ; [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) l'automatise pour les espaces de travail et les projets à plusieurs crates.
- **Le code C dans les dépendances.** rusqlite avec `bundled` compile SQLite : il faut `RUN apk add --no-cache musl-dev` dans l'étape de build. Le `Dockerfile` du labo 16 le fait.
- **Les certificats et les fuseaux.** `scratch` n'a ni `/etc/ssl/certs` ni `/usr/share/zoneinfo`. Avec reqwest et la feature `rustls-tls`, les certificats racines de Mozilla sont embarqués dans le binaire (crate `webpki-roots`) : rien à copier. Avec `native-tls`, il faudrait `COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/`. Pour les fuseaux, la crate `chrono-tz` les embarque aussi. Si tu préfères ne pas y penser, `gcr.io/distroless/static-debian12:nonroot` remplace `scratch` : certificats, fuseaux, `/etc/passwd` et un utilisateur `nonroot`, pour deux méga-octets de plus.

**Venant de Go :** l'image finale est la même, `scratch` ou distroless avec le binaire seul. La différence est dans la première étape : `golang:1.27` et `CGO_ENABLED=0` suffisent en Go ; en Rust, il faut l'image alpine (ou la cible musl), et `musl-dev` dès qu'il y a du C.

**Venant de Python :** `python:3.12-slim` pèse 149 Mo avant ton code, puis viennent le `venv`, les wheels, et le choix entre `slim` et `alpine` selon que tes dépendances ont des binaires pour musl. Ici, 663 Ko, et un `docker run` qui démarre en quelques millisecondes. `uv` a beaucoup simplifié la vie côté Python (et il est écrit en Rust), mais il ne peut rien contre l'interpréteur à embarquer.

### 15.12 La configuration vient de l'environnement

Ce qui change entre ta machine et la production (adresse d'écoute, DSN, niveau de journal) vient de variables d'environnement, jamais d'un fichier commité : les douze facteurs le disent depuis 2011, et Kubernetes, Docker et systemd sont faits pour ça. Avec clap (chapitre 12), chaque option peut venir d'une variable :

```rust
/// Un service configuré par l'environnement.
#[derive(Parser, Debug)]
#[command(name = "service", version)]
struct Config {
    /// Adresse d'écoute.
    #[arg(long, env = "SONDES_ECOUTE", default_value = "127.0.0.1:8080")]
    ecoute: String,
    /// DSN de la base : obligatoire, jamais affiché.
    #[arg(long, env = "SONDES_DSN", hide_env_values = true)]
    dsn: String,
    /// Secondes entre deux tours.
    #[arg(long, env = "SONDES_INTERVALLE", default_value_t = 30)]
    intervalle: u64,
    /// Journal en JSON (pour Loki, Elasticsearch...) plutôt qu'en texte.
    #[arg(long, env = "SONDES_LOG_JSON")]
    log_json: bool,
}
```

```
$ service
error: the following required arguments were not provided:
  --dsn <DSN>

Usage: service --dsn <DSN>

For more information, try '--help'.
$ echo $?
2
$ SONDES_DSN=x SONDES_INTERVALLE=trente service
error: invalid value 'trente' for '--intervalle <INTERVALLE>': invalid digit found in string
```

L'ordre de priorité est le bon d'office : l'option de la ligne de commande, puis la variable, puis la valeur par défaut. Tout est lu et validé au démarrage, avec un message précis et le code 2 : mieux qu'une panne deux heures plus tard à la première requête. Un préfixe par programme (`SONDES_`) évite les collisions. Et `hide_env_values` garde le DSN hors de `--help` ; ne le mets pas non plus dans un `{:?}` de la config que tu journaliserais au démarrage.

### 15.13 Journaliser : tracing et `RUST_LOG`

Un service écrit son journal sur stderr (ou stdout), en lignes, et laisse Docker, systemd ou Kubernetes le collecter. Jamais de fichier de log ouvert par le programme. Le standard de l'écosystème est [tracing](https://docs.rs/tracing) pour écrire, et `tracing-subscriber` pour décider du format et du filtre :

```rust
// RUST_LOG filtre (info par défaut), sur stderr, en texte ou en JSON.
let filtre = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
let journal = tracing_subscriber::fmt().with_env_filter(filtre).with_writer(std::io::stderr);
if config.log_json {
    journal.json().init();
} else {
    journal.with_ansi(false).init();
}

info!(ecoute = %config.ecoute, intervalle_s = config.intervalle, version = env!("CARGO_PKG_VERSION"), "démarrage");
let duree = Duration::from_millis(12);
info!(cible = "db-01:5432", duree_ms = duree.as_millis(), ok = true, "sonde terminée");
warn!(cible = "web-02:443", erreur = "connection refused", "cible injoignable");
tracing::debug!("visible seulement avec RUST_LOG=debug");
```

```
$ SONDES_DSN=x service
2026-10-25T18:32:33.607415Z  INFO service: démarrage ecoute=127.0.0.1:8080 intervalle_s=30 version="1.4.0"
2026-10-25T18:32:33.607493Z  INFO service: sonde terminée cible="db-01:5432" duree_ms=12 ok=true
2026-10-25T18:32:33.607504Z  WARN service: cible injoignable cible="web-02:443" erreur="connection refused"
$ SONDES_DSN=x SONDES_LOG_JSON=true SONDES_INTERVALLE=60 service
{"timestamp":"2026-10-25T18:32:33.611169Z","level":"INFO","fields":{"message":"démarrage","ecoute":"127.0.0.1:8080","intervalle_s":60,"version":"1.4.0"},"target":"service"}
{"timestamp":"2026-10-25T18:32:33.611291Z","level":"INFO","fields":{"message":"sonde terminée","cible":"db-01:5432","duree_ms":"12","ok":true},"target":"service"}
{"timestamp":"2026-10-25T18:32:33.611308Z","level":"WARN","fields":{"message":"cible injoignable","cible":"web-02:443","erreur":"connection refused"},"target":"service"}
$ SONDES_DSN=x RUST_LOG=debug service 2>&1 | tail -1
2026-10-25T18:32:33.614826Z DEBUG service: visible seulement avec RUST_LOG=debug
```

Des champs plutôt que du texte : `cible`, `duree_ms`, `ok` deviennent des clés JSON, et `{cible="db-01:5432"} | json | duree_ms > 100` dans Grafana remplace un `grep` fragile. `%valeur` formate avec `Display`, `?valeur` avec `Debug`. `RUST_LOG` filtre par niveau et par module (`RUST_LOG=info,sondes=debug,tower_http=warn`), sans recompiler.

**Piège :** regarde `duree_ms` dans la sortie JSON : `"12"`, une chaîne, alors que `intervalle_s` est le nombre `60`. `as_millis()` rend un `u128`, que tracing n'enregistre pas comme un nombre, et le JSON reçoit sa représentation texte. Ton filtre `duree_ms > 100` dans Loki ne marchera pas. Convertis en `u64` avant de journaliser (`u64::try_from(d.as_millis()).unwrap_or(u64::MAX)`).

**Venant de Go :** `log/slog` avec `slog.NewJSONHandler` est l'équivalent exact ; les *spans* de tracing (un contexte qui s'attache à tout ce qui est journalisé dedans, comme le `request{method=GET uri=/healthz}` de `TraceLayer` au chapitre 13) sont ce que `slog.With` fait à la main.

### 15.14 Codes de sortie et signaux

Le shell, systemd et Kubernetes lisent ton code de sortie. Les conventions : `0` pour un succès, `1` pour une erreur, `2` pour une mauvaise utilisation (c'est ce que clap rend). Trois façons de sortir en Rust, dans l'ordre de préférence :

- **`fn main() -> anyhow::Result<()>`** : une `Err` affiche `Error: ...` avec la chaîne des causes, et sort avec 1. Le cas courant.
- **`fn main() -> ExitCode`** : `ExitCode::SUCCESS`, `ExitCode::FAILURE`, ou `ExitCode::from(2)` pour un code précis. Les destructeurs tournent normalement avant la sortie (fichiers vidés, transactions annulées).
- **`std::process::exit(n)`** : sort tout de suite, sans appeler les destructeurs de la pile. À réserver aux cas où on sait qu'il n'y a rien à nettoyer, comme le `exit(2)` du labo 14 pour un article inconnu.

Un panic sort avec le code **101**, après le message `thread 'main' panicked at ...` : c'est le code que tu verras dans `systemctl status` quand un `expect` a cédé (avec `panic = "abort"`, le processus est tué par SIGABRT, code 134 dans le shell).

Côté signaux, le chapitre 13 a tout montré : SIGINT et SIGTERM avec `tokio::signal`, et l'arrêt propre d'axum. Deux compléments pour un service. **SIGHUP**, par convention « relis ta configuration » : `signal(SignalKind::hangup())` donne un flux qu'on écoute dans une boucle (c'est une piste du projet final). Et **le délai** : `docker stop` attend dix secondes, Kubernetes trente (`terminationGracePeriodSeconds`), puis SIGKILL, qui ne s'intercepte pas. Ton arrêt propre doit tenir dans ce délai, ou être borné par un `tokio::time::timeout`. Un programme sans async qui veut juste savoir qu'on lui a demandé de s'arrêter peut utiliser la crate `ctrlc`, ou `signal-hook` pour tous les signaux.

### 15.15 Pour le labo

Le [labo 15](../labs/15-release/README.md) amène `hostinfo` au niveau livrable : des fonctions pures testées sur des entrées fixes (`/proc/uptime`, `sysctl kern.boottime`), la version de `Cargo.toml` dans `--version`, puis tous les fichiers de ce chapitre, expliqués un par un : `[lints]`, `rustfmt.toml`, le profil release, un `Makefile` (`build`, `test`, `lint`, `release`, `cross`, `docker`), le `Dockerfile` vers `scratch`, le workflow CI sur trois systèmes, et un `verifier.sh` qui contrôle que tout est en place. Docker n'est pas exigé : si tu l'as, l'image fait 663 Ko.

### À retenir

- `cargo clippy` sans avertissement est la norme ; `pedantic` à bon escient, les règles dans `[lints]` du `Cargo.toml`, `-D warnings` en CI, et `#[expect(..., reason = "...")]` plutôt qu'`#[allow]`.
- `cargo fmt --check` en CI, un `rustfmt.toml` minimal ; les exemples de la documentation sont des tests ; `cargo audit` et `cargo deny` contre les failles connues et les licences.
- La CI : fmt et clippy une fois, tests et build sur Linux, macOS et Windows, avec `Swatinem/rust-cache`.
- Le profil release du livrable : `strip`, `lto = "fat"`, `codegen-units = 1`, et `panic = "abort"` pour un outil (pas forcément pour un serveur) ; `hostinfo` passe de 937 Ko à 553 Ko, et de 3,3 Mo en debug.
- Le binaire statique se demande : cible musl, éditeur de liens `rust-lld` pour du Rust pur, `cargo zigbuild` ou `cross` dès qu'il y a du C.
- La version vient de `Cargo.toml` par `env!("CARGO_PKG_VERSION")` et `#[command(version)]` ; `rust-version` protège ta CI des dépendances trop récentes ; cargo-dist pour les releases.
- Docker : `rust:1.93-alpine` pour compiler, `scratch` ou distroless pour tourner, une couche de dépendances en cache ; 663 Ko pour `hostinfo`, 5,5 Mo pour un service axum avec SQLite.
- Configuration par variables d'environnement validées au démarrage, journal tracing sur stderr filtré par `RUST_LOG`, codes de sortie 0, 1, 2 (et 101 pour un panic), arrêt propre dans le délai de SIGTERM.

---

← [14. Bases de données : SQLite et PostgreSQL](14-bases-de-donnees.md) · [Sommaire](../README.md) · [16. Projet final et pour aller plus loin](16-projet-final-et-suite.md) →
