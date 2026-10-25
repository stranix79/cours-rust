# 1. Installer, outiller, lancer

*Rust de zéro à la prod : chapitre 1 sur 16.* ← [0. Pourquoi Rust, et quand l'utiliser (ou pas)](00-pourquoi-rust.md) · [Sommaire](../README.md) · [2. Variables, types, chaînes](02-variables-types-chaines.md) →

L'outillage Rust tient en deux commandes. `rustup` installe et met à jour le compilateur, ses outils et ses cibles. `cargo` fait tout le reste : créer un projet, télécharger les dépendances, compiler, lancer, tester, formater, documenter. Pas de `Makefile` à écrire, pas d'environnement virtuel, pas de débat sur le système de build : tout le monde utilise Cargo, et un projet Rust pris au hasard sur GitHub se compile avec `cargo build`. Ce chapitre installe tout, écrit un premier programme, et explique ce que chaque commande fait vraiment, parce que tu vas les taper cent fois par jour.

### 1.1 Installer avec `rustup`

On n'installe pas Rust avec le gestionnaire de paquets du système. On installe `rustup`, qui installe Rust :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Tout atterrit dans ton dossier personnel : `~/.rustup` pour les compilateurs, `~/.cargo/bin` pour les commandes (`rustc`, `cargo`, `rustup`, `rustfmt`, `cargo-clippy`). Rien dans `/usr`, rien qui demande `sudo`. Le script ajoute `~/.cargo/bin` à ton `PATH` ; ouvre un nouveau terminal et vérifie :

```
$ rustc -V
rustc 1.93.1 (01f6ddf75 2026-02-11)
$ cargo -V
cargo 1.93.1 (083ac5135 2025-12-15)
```

Pourquoi pas `apt install rustc` ? Parce que Rust sort une version toutes les six semaines, que les crates récentes demandent souvent un compilateur récent, et que la version de ta distribution aura un an de retard. `rustup update` te met à jour en une commande, sur Linux, macOS et Windows de la même façon.

`rustup` gère trois choses, qu'il vaut mieux distinguer tout de suite.

**Les toolchains.** Une toolchain est un compilateur complet avec ses outils, dans une version donnée. Il y a trois canaux : `stable` (celui que tu utilises, et le seul de ce cours), `beta` (la prochaine stable, dans six semaines) et `nightly` (construite chaque nuit, avec les fonctionnalités expérimentales). Tu peux en avoir plusieurs côte à côte et choisir par projet :

```bash
rustup toolchain install nightly   # pour essayer, jamais pour la prod
rustup override set 1.85.0         # ce dossier utilise la 1.85.0
cargo +nightly build               # une seule commande avec une autre toolchain
```

Un fichier `rust-toolchain.toml` à la racine d'un dépôt fixe la toolchain pour tout le monde. C'est l'équivalent de la ligne `go 1.25` d'un `go.mod`, en plus strict : rustup installe la bonne version tout seul si elle manque.

**Les composants.** Les pièces d'une toolchain : `rustc`, `cargo`, `rust-std` (la bibliothèque standard précompilée), `clippy`, `rustfmt`, `rust-docs` (la documentation hors ligne), `rust-analyzer`, `rust-src` (les sources de la std, pour que l'éditeur puisse t'y emmener). Le profil d'installation par défaut met déjà les premiers. Les autres s'ajoutent :

```bash
rustup component add rust-analyzer rust-src
```

**Les cibles.** Le compilateur Rust sait produire du code pour une centaine de plateformes, mais il faut la bibliothèque standard précompilée pour chacune. Une cible s'appelle par un *triplet* : architecture, fabricant, système, et parfois l'ABI. `aarch64-apple-darwin` est un Mac Apple Silicon, `x86_64-unknown-linux-gnu` un Linux classique avec la glibc, `x86_64-unknown-linux-musl` un Linux avec la libc musl (binaires statiques, on y revient en 1.10), `x86_64-pc-windows-msvc` un Windows.

```
$ rustup show
Default host: aarch64-apple-darwin
rustup home:  /Users/stranix/.rustup

installed toolchains
--------------------
stable-aarch64-apple-darwin (active, default)

active toolchain
----------------
name: stable-aarch64-apple-darwin
active because: it's the default toolchain
installed targets:
  aarch64-apple-darwin
```

Ici, une seule toolchain et une seule cible : celle de la machine. `rustup target list` affiche les cent autres.

### 1.2 Le premier programme

```
$ cargo new bonjour
    Creating binary (application) `bonjour` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
$ cd bonjour
$ ls -a
.  ..  .git  .gitignore  Cargo.toml  src
```

`cargo new` crée un dossier, un dépôt Git (avec un `.gitignore` qui contient `/target`), un fichier `Cargo.toml` et un fichier `src/main.rs`. C'est tout ce qu'il faut. Le `Cargo.toml` :

```toml
[package]
name = "bonjour"
version = "0.1.0"
edition = "2024"

[dependencies]
```

Et le programme :

```rust
fn main() {
    println!("Hello, world!");
}
```

```
$ cargo run
   Compiling bonjour v0.1.0 (…/bonjour)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.32s
     Running `target/debug/bonjour`
Hello, world!
```

Trois lignes de Cargo, puis la sortie du programme. `Compiling` : Cargo appelle le compilateur `rustc` sur ton code. `Finished` : il dit avec quel profil (ici `dev`, non optimisé, avec les informations de débogage). `Running` : il lance le binaire qu'il vient de produire, `target/debug/bonjour`. Relance `cargo run` sans rien changer : la ligne `Compiling` disparaît, Cargo sait que rien n'a bougé.

**Venant de Go :** `cargo new` est `go mod init` plus le `main.go`, `cargo run` est `go run .`, `cargo build` est `go build`. Le `Cargo.toml` joue le rôle du `go.mod`, et le `Cargo.lock` (1.5) celui du `go.sum`, en plus précis : il fixe la version exacte de chaque dépendance, pas seulement son empreinte. Une différence de culture : Go compile en une fraction de seconde et on ne pense jamais au temps de build ; en Rust, on y pense (1.11).

### 1.3 Anatomie de `fn main` et de `println!`

Le programme tient en trois lignes, et chacune dit quelque chose du langage.

`fn main()` déclare une fonction sans paramètre et sans valeur de retour. `main` est le point d'entrée d'un exécutable, comme en C et en Go. Pas de `package main`, pas d'`import` obligatoire, pas de `int argc, char **argv` : les arguments se lisent avec `std::env::args()`, au chapitre 12. Les accolades sont obligatoires, l'indentation est de quatre espaces (rustfmt y veille), et chaque instruction se termine par un point-virgule.

`println!` n'est pas une fonction. Le point d'exclamation dit que c'est une **macro** : du code qui s'exécute *pendant la compilation* et produit du code Rust. Pourquoi une macro pour afficher une ligne ? Parce qu'une fonction Rust a un nombre fixe de paramètres, d'un type fixé, et que `println!` doit accepter n'importe quel nombre d'arguments de n'importe quels types. Et surtout parce que la macro lit la chaîne de format à la compilation et vérifie qu'elle correspond aux arguments :

```rust
fn main() {
    let hote = "db01";
    let port = 5432;
    println!("Connexion à {}:{}", hote, port);
    println!("Connexion à {hote}:{port}");
    let s = format!("{hote}:{port}");
    println!("{s} fait {} octets", s.len());
}
```

```
$ cargo run
Connexion à db01:5432
Connexion à db01:5432
db01:5432 fait 9 octets
```

Chaque `{}` est remplacé par un argument, dans l'ordre. `{hote}` va chercher directement la variable `hote` : c'est l'argument en ligne, la forme que tu écriras le plus. `format!` fait la même chose mais rend une `String` au lieu d'afficher. Le chapitre 2 fait le tour des options (`{:?}`, `{:<10}`, `{:.2}`). Et si tu te trompes dans le nombre d'arguments :

```rust
println!("{:<10}: {}", "Machine");
```

```
$ cargo build
error: 2 positional arguments in format string, but there is 1 argument
 --> src/main.rs:3:15
  |
3 |     println!("{:<10}: {}", "Machine");
  |               ^^^^^^  ^^   ---------
```

L'erreur arrive avant l'exécution, avec la position exacte des deux `{}` et du seul argument. C'est un bon résumé de Rust : ce que les autres langages vérifient à l'exécution (ou pas du tout), Rust essaie de le vérifier à la compilation.

**Venant du C :** `printf("%d", "oups")` compile en C (au mieux avec un avertissement) et affiche n'importe quoi. Le même mélange en Rust ne compile pas : la macro connaît le type de chaque argument, et un type qui ne sait pas s'afficher est refusé. Pas de `%d`, `%s`, `%lu` à choisir non plus : `{}` marche pour tout ce qui sait s'afficher.

**Venant de Python :** `println!("{hote}:{port}")` est ta f-string `f"{hote}:{port}"`, vérifiée à la compilation. On ne peut pas y mettre d'expression (`{port + 1}` ne marche pas) : seulement un nom de variable, sinon on passe l'expression en argument, `println!("{}", port + 1)`.

### 1.4 `build`, `run`, `check`, `test` : où va le binaire

Les quatre commandes de tous les jours :

| Commande | Ce qu'elle fait |
|---|---|
| `cargo build` | compile, profil `dev`, binaire dans `target/debug/` |
| `cargo build --release` | compile optimisé, profil `release`, binaire dans `target/release/` |
| `cargo run` | `build` puis lance le binaire ; les arguments du programme vont après `--` : `cargo run -- --port 8080` |
| `cargo check` | vérifie que tout compile, sans produire de binaire : deux à cinq fois plus rapide |
| `cargo test` | compile en mode test et lance tous les tests du paquet |

`cargo check` est celle qu'on oublie et qu'on devrait taper le plus : quand tu corriges des erreurs de compilation, tu n'as pas besoin d'un binaire, juste de savoir si le compilateur est content. Ton éditeur la lance pour toi à chaque sauvegarde (1.8).

Les deux **profils** n'ont pas les mêmes réglages, et la différence est grande :

```
$ cargo build --release
   Compiling bonjour v0.1.0 (…/bonjour)
    Finished `release` profile [optimized] target(s) in 0.21s
$ ls -la target/debug/bonjour target/release/bonjour
-rwxr-xr-x@ 1 stranix  wheel  446152 25 oct.  18:26 target/debug/bonjour
-rwxr-xr-x@ 1 stranix  wheel  407232 25 oct.  18:26 target/release/bonjour
$ file target/release/bonjour
target/release/bonjour: Mach-O 64-bit executable arm64
```

Le profil `dev` compile vite, n'optimise rien, garde les symboles de débogage, et **vérifie les débordements d'entiers** : un `u8` qui dépasse 255 fait paniquer le programme (chapitre 2). Le profil `release` optimise (`opt-level = 3`), compile plus lentement, et enroule les débordements en silence. Un programme en `dev` peut être dix ou cinquante fois plus lent qu'en `release` sur du calcul. **Règle : on mesure les performances en `--release`, jamais autrement**, et on livre toujours un binaire `--release`.

Les profils se règlent dans `Cargo.toml`. Pour un binaire que tu distribues, ces trois lignes sont classiques :

```toml
[profile.release]
strip = true        # retire les symboles du binaire
lto = true          # optimise à travers toutes les crates, à l'édition de liens
codegen-units = 1   # compile en un seul morceau : plus lent, mieux optimisé
```

```
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 2.41s
$ ls -la target/release/bonjour
-rwxr-xr-x@ 1 stranix  wheel  302608 25 oct.  18:27 target/release/bonjour
```

De 407 Ko à 303 Ko pour un « Hello, world », et l'écart grandit avec les dépendances. Trois cents kilo-octets peut sembler gros pour afficher une ligne : le binaire embarque la partie de la bibliothèque standard qu'il utilise (le formatage, la gestion des paniques, les entrées-sorties), et il ne dépend de rien d'autre que de la libc du système. C'est le même modèle que Go, en plus compact.

Le dossier `target/` contient tout ce que Cargo produit : les binaires, les dépendances compilées (`deps/`), le cache de compilation incrémentale (`incremental/`). Il grossit vite (un ou deux giga-octets pour un projet avec `tokio` et `axum`), il ne se commite jamais, et `cargo clean` le vide sans risque.

### 1.5 `Cargo.toml`, `Cargo.lock`, crates.io et docs.rs

Une bibliothèque Rust publiée s'appelle une **crate**. Le dépôt public est [crates.io](https://crates.io), et la documentation de chaque crate publiée est générée et hébergée sur [docs.rs](https://docs.rs). Ajouter une dépendance :

```
$ cargo add rand@0.9
    Updating crates.io index
      Adding rand v0.9 to dependencies
             Features as of v0.9.0:
             + alloc
             + os_rng
             + small_rng
             + std
             + std_rng
             + thread_rng
             - log
             - nightly
             - serde
             - simd_support
             - unbiased
```

`cargo add` écrit une ligne dans `Cargo.toml` :

```toml
[dependencies]
rand = "0.9"
```

`"0.9"` ne veut pas dire « exactement 0.9 » : c'est une contrainte semver, « toute version compatible avec 0.9 », donc `0.9.x`. Les **features** sont des options de compilation de la crate : les `+` sont actives par défaut, les `-` s'activent avec `cargo add rand --features serde`. Elles servent à ne compiler que ce dont tu as besoin, et tu les retrouveras partout (`tokio` avec `features = ["full"]`, `reqwest` avec `rustls-tls`).

Au premier `cargo build`, Cargo choisit la version la plus récente qui respecte la contrainte, la télécharge, la compile, et écrit le choix exact dans **`Cargo.lock`** :

```toml
[[package]]
name = "rand"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9ef1d0d795eb7d84685bca4f72f3649f064e6641543d3a8c415898726a57b41"
```

Les builds suivants utilisent `0.9.5` et rien d'autre, jusqu'à ce que tu lances `cargo update`. **Commite le `Cargo.lock`** : c'est lui qui garantit que ton binaire compilé en CI dans six mois utilise les mêmes versions que sur ta machine aujourd'hui.

Le champ `edition = "2024"` mérite une explication. Une **édition** est un ensemble de règles du langage, choisi par crate : 2015, 2018, 2021 et 2024. Une nouvelle édition peut changer des détails (de nouveaux mots-clés réservés, la portée de certaines variables temporaires), mais une crate en édition 2021 se compile avec un compilateur récent et se mélange sans souci avec des crates en 2024. C'est la façon qu'a Rust d'évoluer sans casser le code existant, et de ne pas refaire l'histoire de Python 2 et 3. Ce cours est en édition 2024, la plus récente ; `cargo new` la choisit par défaut.

**Venant de Python :** `pip`, `venv`, `requirements.txt`, `pyproject.toml`, `poetry.lock` : tout ça est remplacé par `Cargo.toml` et `Cargo.lock`. Il n'y a pas d'environnement virtuel à activer, parce qu'il n'y a pas d'installation globale des dépendances : chaque projet compile les siennes dans son `target/`, dans la version de son `Cargo.lock`. Deux projets peuvent utiliser deux versions incompatibles de la même crate sans se gêner. Si `uv` t'a plu, c'est normal : il a copié le modèle de Cargo, et il est écrit en Rust.

**Venant de Go :** pas de `GOPATH`, pas de `go get` qui modifie un cache global, mais un cache tout de même (`~/.cargo/registry`) pour ne pas retélécharger. La grosse différence est que les dépendances Go se compilent en un éclair et que les dépendances Rust prennent du temps la première fois (1.11). Et `go.sum` ne fait que vérifier des empreintes, alors que `Cargo.lock` choisit les versions.

### 1.6 L'espace de travail de ce cours

Ce dépôt contient seize labos, et chaque labo est un paquet Cargo indépendant, avec son `Cargo.toml`, et une solution qui est un second paquet. Si chacun avait son propre `target/`, une dépendance comme `tokio` serait compilée trente fois, et le dépôt pèserait des dizaines de giga-octets. D'où un **espace de travail** (*workspace*), déclaré par le `Cargo.toml` à la racine :

```toml
[workspace]
resolver = "3"
members = ["labs/*", "labs/*/solution"]
```

Tous les paquets listés partagent un seul `Cargo.lock` et un seul `target/`, à la racine. Conséquences pratiques :

- `cargo run` lancé dans `labs/01-hello/` ne compile que ce labo, mais le binaire atterrit dans `../../target/debug/hello`, pas dans `labs/01-hello/target/`. C'est ce que montrent les consignes du [labo 01](../labs/01-hello/README.md).
- Une dépendance compilée pour un labo est réutilisée par les autres, tant que la version et les features sont les mêmes.
- `resolver = "3"` choisit l'algorithme de résolution des versions de l'édition 2024 : il tient compte de la version minimale de Rust que chaque crate déclare.

Tu n'auras pas besoin d'espace de travail pour tes petits outils. Tu le retrouveras dès qu'un projet grandit : un dépôt avec une bibliothèque, une CLI et un serveur qui la partagent est presque toujours un workspace (chapitre 9).

### 1.7 `rustfmt`, `clippy`, `cargo doc`

Trois outils livrés avec la toolchain, à lancer avant chaque commit.

**`cargo fmt`** reformate tout le code du paquet selon le style officiel. Il n'y a rien à configurer, et il ne faut rien configurer : tout le code Rust du monde a la même allure, ce qui rend lisible le code des autres. `cargo fmt --check` ne modifie rien et affiche ce qui changerait, c'est la version pour la CI :

```
$ cargo fmt --check
Diff in …/bonjour/src/main.rs:4:
         println!("port {}", ports[i]);
     }
     let actif = true;
-    if actif == true { println!("actif") }
+    if actif == true {
+        println!("actif")
+    }
 }
```

**`cargo clippy`** est un linter : plus de 700 règles qui repèrent du code correct mais maladroit, non idiomatique, lent ou suspect. Le même petit programme :

```rust
fn main() {
    let ports = vec![22, 80, 443];
    for i in 0..ports.len() {
        println!("port {}", ports[i]);
    }
    let actif = true;
    if actif == true { println!("actif") }
}
```

```
$ cargo clippy
warning: the loop variable `i` is only used to index `ports`
 --> src/main.rs:3:14
  |
3 |     for i in 0..ports.len() {
  |              ^^^^^^^^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#needless_range_loop
help: consider using an iterator
  |
3 -     for i in 0..ports.len() {
3 +     for <item> in &ports {
  |

warning: equality checks against true are unnecessary
 --> src/main.rs:7:8
  |
7 |     if actif == true { println!("actif") }
  |        ^^^^^^^^^^^^^ help: try: `actif`

warning: useless use of `vec!`
 --> src/main.rs:2:17
  |
2 |     let ports = vec![22, 80, 443];
  |                 ^^^^^^^^^^^^^^^^^ help: you can use an array directly: `[22, 80, 443]`
```

Trois remarques, trois façons d'écrire mieux, chacune avec la correction proposée et un lien qui explique la règle. Quand tu débutes, clippy est un professeur patient : lis ses messages, ils t'apprennent l'idiome plus vite qu'un livre. En CI, on le lance avec `cargo clippy --all-targets -- -D warnings`, qui transforme tout avertissement en erreur et couvre aussi les tests ; c'est ce que fait le vérificateur de ce dépôt.

**`cargo doc --open`** génère la documentation HTML de ton projet *et de toutes ses dépendances*, dans la version exacte de ton `Cargo.lock`, et l'ouvre dans le navigateur. Hors ligne, à jour, avec la recherche. Pour la bibliothèque standard, `rustup doc --std` ouvre la copie locale de [doc.rust-lang.org/std](https://doc.rust-lang.org/std/). Les commentaires `///` au-dessus d'une fonction deviennent sa documentation, en Markdown ; le chapitre 9 y revient.

**Venant de Go :** `cargo fmt` est `gofmt`, même philosophie : un seul style, pas d'options. `cargo clippy` est `go vet` en beaucoup plus bavard : `go vet` cherche des bugs probables, clippy cherche aussi le code pas idiomatique. `cargo doc` est `go doc` avec une vraie page HTML, et docs.rs est pkg.go.dev.

### 1.8 L'éditeur : rust-analyzer

Écrire du Rust sans rust-analyzer, c'est écrire du Go sans gopls : possible, pénible. rust-analyzer est le serveur de langage officiel. Il lance `cargo check` à chaque sauvegarde et affiche les erreurs dans le code, montre le type déduit de chaque variable (les *inlay hints* en gris, précieux en Rust où l'inférence fait beaucoup), complète les méthodes, renomme, et t'emmène dans les sources de la std sur un clic.

- **VS Code** : l'extension « rust-analyzer » (celle de l'organisation rust-lang, pas l'ancienne extension « Rust »). Elle télécharge son propre serveur.
- **Zed** : le support Rust est intégré, rien à installer. Zed est lui-même écrit en Rust, et ça se sent.
- **Vim, Neovim, Helix, Emacs** : n'importe quel client LSP, avec le serveur installé par `rustup component add rust-analyzer`.

Un réglage qui change la vie dans VS Code : `"rust-analyzer.check.command": "clippy"`, pour voir les remarques de clippy en direct, pas seulement les erreurs du compilateur.

### 1.9 Binaire ou bibliothèque

`cargo new bonjour` crée un **binaire** : un `src/main.rs` avec un `fn main`. `cargo new --lib outils` crée une **bibliothèque** : un `src/lib.rs` sans `main`, avec un exemple de test.

```rust
/// Renvoie l'adresse `hote:port`.
pub fn adresse(hote: &str, port: u16) -> String {
    format!("{hote}:{port}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adresse_simple() {
        assert_eq!(adresse("db01", 5432), "db01:5432");
    }
}
```

```
$ cargo test
   Compiling outils v0.1.0 (…/outils)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running unittests src/lib.rs (target/debug/deps/outils-52a77812af069ca3)

running 1 test
test tests::adresse_simple ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests outils

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`pub` rend la fonction visible hors de la crate ; sans lui, elle est privée. Les tests vivent dans le même fichier, dans un module `tests` qui n'est compilé que pour `cargo test` (`#[cfg(test)]`). Les « Doc-tests » sont les exemples de code écrits dans les commentaires `///`, que `cargo test` compile et exécute aussi : la documentation ne peut pas mentir (chapitre 9).

Un paquet peut contenir les deux, et c'est la structure de tous les labos à partir du 02 : `src/lib.rs` contient la logique, `src/main.rs` est un petit programme qui l'appelle, et `tests/` contient des tests qui utilisent la bibliothèque comme le ferait un autre programme. Le binaire importe la bibliothèque par son nom : `use convertisseur::taille_lisible;`. Pourquoi ce découpage ? Parce qu'un `main` qui lit le clavier ou le réseau se teste mal, alors qu'une fonction qui prend des paramètres et rend un résultat se teste en une ligne. Le cours Go faisait la même chose.

### 1.10 Compiler pour une autre machine

Ton Mac est un `aarch64-apple-darwin`, ton serveur un `x86_64-unknown-linux-gnu`. Rust compile en croisé avec `--target`, après avoir installé la bibliothèque standard de la cible :

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
file target/x86_64-unknown-linux-musl/release/bonjour
```

Le binaire va dans `target/<triplet>/release/`. Pourquoi `musl` plutôt que `gnu` ? Un binaire lié à la glibc dépend de la version de la glibc du système où il tourne : compilé sur une Debian récente, il refuse de démarrer sur une vieille CentOS. Un binaire `musl` est **statique** : il embarque sa libc et tourne sur n'importe quel Linux x86-64, y compris dans une image Docker `scratch` vide. C'est le format de livraison idéal pour un outil de sysadmin.

Il y a une différence avec Go, et elle est honnête : Go compile en croisé sans rien d'autre que `GOOS=linux GOARCH=amd64 go build`, parce qu'il a son propre éditeur de liens et n'appelle pas de libc. Rust utilise l'éditeur de liens du système (`cc`, `ld`), et ton Mac n'a pas d'éditeur de liens pour Linux. La compilation passe, l'édition de liens échoue. Les solutions, vues au chapitre 15 : `cargo zigbuild` (qui utilise Zig comme éditeur de liens croisé, une installation et ça marche), `cross` (qui compile dans un conteneur Docker), ou simplement compiler en CI sur la bonne plateforme.

**Venant de Go :** `GOOS` et `GOARCH` deviennent un triplet passé à `--target`. `CGO_ENABLED=0` pour un binaire statique devient la cible `musl`. Et `runtime.GOOS` devient `std::env::consts::OS`, une constante fixée à la compilation, que le [labo 01](../labs/01-hello/README.md) affiche.

**Venant du C :** tu connais le problème : une chaîne de compilation croisée, c'est un compilateur, des en-têtes, une libc et un éditeur de liens pour la cible. Rust te fournit les trois premiers avec `rustup target add`, il ne reste que l'éditeur de liens. Et `cargo` remplace `make`, `./configure`, `pkg-config` et le `Makefile` que tu recopiais de projet en projet : il sait dans quel ordre compiler, quoi recompiler, et où trouver les dépendances.

### 1.11 Un mot honnête sur les temps de compilation

Rust compile lentement. Pas dramatiquement, mais sensiblement plus que Go, et il vaut mieux le savoir avant de s'en plaindre. Sur un MacBook M1 Pro, le petit jeu du labo 03, avec sa seule dépendance `rand` :

```
$ cargo build                      # premier build, rand compris
   Compiling rand v0.9.5
   Compiling randtest v0.1.0 (…/randtest)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.67s
$ touch src/main.rs && cargo build # seul ton code est recompilé
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.19s
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 4.90s
```

Quatre secondes la première fois, deux dixièmes ensuite. Pour un service web avec `tokio`, `axum`, `serde` et `sqlx`, le premier build prend une à deux minutes, et une modification de ton code entre deux et dix secondes. Pourquoi ? Parce que le compilateur fait beaucoup de travail : il vérifie les emprunts (chapitre 4), il génère une version spécialisée de chaque fonction générique pour chaque type utilisé (chapitre 7), et LLVM optimise derrière. C'est le prix des abstractions à coût nul et des garanties.

Ce qui aide, dans l'ordre : `cargo check` au lieu de `cargo build` pendant que tu corriges (l'éditeur le fait pour toi) ; ne pas lancer `cargo clean` sans raison, la compilation incrémentale fait son travail ; un espace de travail pour partager le `target/` entre paquets, comme dans ce dépôt ; et n'ajouter que les features dont tu as besoin. Le jour où tu attends vraiment, le chapitre 15 parle de `sccache` et de l'éditeur de liens `mold`.

### 1.12 Pour le labo

Le [labo 01](../labs/01-hello/README.md) est un `hello` qui dit sur quelle machine il tourne : nom de l'hôte (en lançant la commande `hostname`, la std n'a pas de fonction pour ça), système et architecture, nombre de cœurs, chemin du binaire. Tu y lances `cargo run`, `cargo clippy` et `cargo fmt --check`, tu compiles en `--release`, tu ajoutes le profil `strip`, `lto`, `codegen-units`, et tu compiles pour Linux avec `--target x86_64-unknown-linux-musl`. Pas de tests pour ce premier labo : il n'y a pas de logique à tester, seulement des outils à prendre en main. Les tests arrivent au labo 02.

### À retenir

- `rustup` installe et met à jour les toolchains (`stable`, `beta`, `nightly`), les composants (`clippy`, `rustfmt`, `rust-analyzer`) et les cibles (`rustup target add <triplet>`), le tout dans ton dossier personnel.
- `cargo new` (binaire) ou `cargo new --lib` (bibliothèque), puis `cargo run`, `cargo build`, `cargo check`, `cargo test`.
- `println!` est une macro : elle vérifie la chaîne de format à la compilation. `{}` pour un argument, `{nom}` pour une variable en ligne, `format!` pour obtenir une `String`.
- Profil `dev` dans `target/debug/` (rapide à compiler, débordements vérifiés), profil `release` dans `target/release/` (optimisé). On mesure et on livre en `--release`.
- `Cargo.toml` déclare les dépendances avec une contrainte semver, `Cargo.lock` fixe les versions exactes : commite les deux. Les crates viennent de crates.io, leur documentation de docs.rs ou de `cargo doc --open`.
- L'édition (`2024`) se choisit par crate et permet au langage d'évoluer sans casser le code existant.
- Ce cours est un espace de travail : un seul `target/` et un seul `Cargo.lock` à la racine, pour tous les labos.
- Avant chaque commit : `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. Lis les remarques de clippy, elles t'apprennent l'idiome.
- `--target x86_64-unknown-linux-musl` donne un binaire statique pour tout Linux ; l'éditeur de liens croisé est la seule pièce qui manque sur un Mac.
- Rust compile plus lentement que Go : `cargo check` pendant le travail, et laisse la compilation incrémentale faire son travail.

---

← [0. Pourquoi Rust, et quand l'utiliser (ou pas)](00-pourquoi-rust.md) · [Sommaire](../README.md) · [2. Variables, types, chaînes](02-variables-types-chaines.md) →
