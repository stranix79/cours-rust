# Labo 01 : Bonjour, machine

**Chapitre** : [1. Installer, outiller, lancer](../../cours/01-installer-et-outiller.md)

## Objectif
Écrire, lancer, compiler en mode release et compiler en croisé ton premier programme Rust : un `hello` qui dit sur quelle machine et quel système il tourne. À la fin tu sais ce que `cargo` fait pour toi, où va le binaire, et comment en livrer un pour un serveur Linux.

## Consignes
1. Lis `src/main.rs`. Il compile et se lance tel quel (`cargo run`), mais il n'affiche que la première ligne. Regarde `Cargo.toml` : le nom du paquet, l'édition, et un bloc `[[bin]]` qui nomme le binaire `hello` (sans lui, il s'appellerait `lab01-hello`). Regarde aussi le `Cargo.toml` à la racine du dépôt : c'est l'espace de travail qui regroupe tous les labos dans un seul dossier `target/`.
2. Complète les `// TODO` : le nom de la machine (pas de `hostname()` dans la bibliothèque standard, on lance la commande `hostname` avec `std::process::Command`), le système et l'architecture (`std::env::consts::OS` et `ARCH`), le nombre de cœurs (`std::thread::available_parallelism()`) et le chemin du binaire (`std::env::current_exe()`). Chaque `TODO` donne la piste ; le chapitre 1 montre les `use`.
3. Lance `cargo run`, puis `cargo clippy` : aucun message, c'est bon signe. Lance `cargo fmt --check` : s'il affiche un diff, ton fichier n'est pas formaté, corrige avec `cargo fmt` et regarde ce qui a changé.
4. Compile pour de vrai : `cargo build --release` puis `../../target/release/hello`. Regarde la taille du binaire avec `ls -la` et sa nature avec `file`. Compare avec `cargo build` (sans `--release`) : le binaire de debug est plus gros et plus lent, c'est normal, il contient les vérifications de débordement et les symboles.
5. Compile pour un serveur Linux : `rustup target add x86_64-unknown-linux-musl` une fois, puis `cargo build --release --target x86_64-unknown-linux-musl`. Si l'éditeur de liens manque sur ton Mac, le chapitre 15 montre `cargo zigbuild` et `cross` ; retiens juste qu'un binaire `musl` est statique et tourne sur n'importe quel Linux. Vérifie avec `file`.
6. Bonus : ajoute dans `Cargo.toml` un profil `[profile.release]` avec `strip = true`, `lto = true` et `codegen-units = 1`, recompile en release et compare les tailles. C'est ce qu'on fait pour les binaires qu'on distribue.

## Comment lancer
```bash
cargo run                        # ton programme
(cd solution && cargo run)       # la version corrigée, commentée
cargo clippy && cargo fmt --check
cargo build --release && ../../target/release/hello
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl && file ../../target/x86_64-unknown-linux-musl/release/hello
```

## Sortie attendue
Les valeurs dépendent de ta machine, la forme non :
```
$ cargo run
   Compiling lab01-hello v0.1.0 (…/cours-rust/labs/01-hello)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s
     Running `…/cours-rust/target/debug/hello`
Bonjour depuis Rust.
Machine   : mbp-stranix.local
Système   : macos/aarch64
Cœurs     : 8
Binaire   : /Users/stranix/git/stranix/cours-rust/target/debug/hello
```

## Pour aller plus loin
- `rustup target list` affiche toutes les cibles connues. Compte-les. Cherche `riscv64gc-unknown-linux-gnu` et `wasm32-wasip2`.
- `cargo build -v` montre la commande `rustc` complète que Cargo lance. Tu y verras les flags d'édition, de débogage et le dossier de sortie : Cargo n'est qu'un chef d'orchestre au-dessus de `rustc`.
- Retire volontairement un argument d'un `println!("{:<10}: {}", "Machine")` et compile : l'erreur arrive avant l'exécution, avec la position exacte. C'est le genre de filet que Rust tend partout, et le chapitre 4 montre le plus serré d'entre eux.
