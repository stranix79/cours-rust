# Labo 08 : Robuste

**Chapitre** : [8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic](../../cours/08-erreurs.md)

## Objectif
Écrire un lecteur de fichier de configuration `cle = valeur` qui ne panique jamais et dit toujours pourquoi il refuse : quel fichier, quelle ligne, quelle clé, et la cause d'origine. La bibliothèque expose un enum `ErreurConfig` écrit avec `thiserror`, dont chaque variante se teste avec `matches!` ; le binaire utilise `anyhow` pour ajouter du contexte, affiche la chaîne complète sur stderr avec `{:#}`, et sort avec le code 2 quand la configuration est en cause. À la fin, tu sais choisir entre `#[from]` et `#[source]` plus `map_err`, mettre n'importe quelle cause dans une `Box<dyn Error + Send + Sync>`, et écrire le `main` d'un outil qui se comporte bien dans un script shell.

Crates : `thiserror` et `anyhow`, introduites au chapitre 8 (déjà dans `Cargo.toml`).

## Consignes
1. Lis `src/lib.rs`, `src/main.rs`, et les deux fichiers fournis : `exemple.conf` (valide) et `invalide.conf` (le port de la ligne 3 n'est pas un nombre). Ils existent aussi dans `solution/` : les tests les cherchent dans le dossier du paquet (`env!("CARGO_MANIFEST_DIR")`). Lance `cargo test` : les dix tests échouent, mais ça compile.
2. Les messages d'`ErreurConfig` (TODO 1) : remplace chaque `#[error("TODO")]`. Les textes exacts sont dans les tests, par exemple `ligne 3 : valeur invalide pour « port »` ou `clé obligatoire absente : « port »`. Le message d'une variante ne répète jamais sa cause : c'est `source()` qui la donne.
3. `valeur_port` (TODO 2) et `valeur_hotes` (TODO 3) : deux petites fonctions qui rendent `Result<_, Cause>`, où `Cause` est une `Box<dyn Error + Send + Sync>`. Une `ParseIntError` y entre avec `?`, un texte avec `"...".into()`. Retire ensuite les `#[allow(dead_code)]`.
4. `parser(texte)` (TODO 4, 5 et 6) : la boucle sur les lignes, le `match` sur les clés, puis la `Config` finale avec ses valeurs par défaut. Chaque erreur est construite avec son numéro de ligne (en partant de 1). `cargo test ligne` puis `cargo test cle` puis `cargo test valeurs`.
5. `charger(chemin)` (TODO 7) : `read_to_string`, et un `map_err` qui range le chemin et l'`io::Error` dans `ErreurConfig::Io`. Pourquoi pas `#[from]` ? Parce qu'un `From<io::Error>` ne saurait pas quel chemin mettre. `cargo test fichier`.
6. Le binaire, `run` (TODO 8) : `charger(&chemin).with_context(...)?`, puis l'affichage. Et `main` (TODO 9) : `{e:#}` au lieu de `{e}`, et le code de sortie 2 si `e.downcast_ref::<ErreurConfig>()` trouve une erreur de configuration dans la chaîne. Lance les quatre commandes de la sortie attendue et compare, codes de sortie compris (`echo $?`).
7. `cargo test` vert, puis `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                          # tes tests (rouge tant que les TODO restent)
cargo test valeurs                  # seulement les tests dont le nom contient valeurs
cargo run -- exemple.conf           # ton programme
cargo run -- invalide.conf; echo $? # une erreur de valeur, code 2
(cd solution && cargo test)         # la solution (vert)
(cd solution && cargo run -- exemple.conf)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run -- exemple.conf
port    : 5432
timeout : 3s
hotes   : db-01, db-02, cache-01
verbeux : oui
$ cargo run -- invalide.conf ; echo "rc=$?"
erreur : configuration de la sonde (invalide.conf): ligne 3 : valeur invalide pour « port »: invalid digit found in string
rc=2
$ cargo run -- absent.conf ; echo "rc=$?"
erreur : configuration de la sonde (absent.conf): lecture de absent.conf: No such file or directory (os error 2)
rc=2
$ cargo run ; echo "rc=$?"
usage : robuste FICHIER.conf
rc=2
```

Lis la deuxième ligne de gauche à droite : ce que faisait le programme, où dans le fichier, ce qui n'allait pas, la cause d'origine. Chaque couche n'a ajouté que ce qu'elle savait.

## Pour aller plus loin
- `parser` s'arrête à la première faute. Écris `parser_tout(texte) -> Result<Config, Vec<ErreurConfig>>` qui continue après une ligne invalide et rend toutes les erreurs ; `main` les affiche une par ligne. Qu'est-ce qui change pour les clés manquantes ?
- Accepte des durées avec unité (`timeout = 1500ms`, `timeout = 2s`) : une fonction `valeur_duree` avec `strip_suffix`, et un message de cause qui cite les unités acceptées.
- Le chapitre 9 transforme cette bibliothèque en vraie crate : documentation avec des exemples testés (`cargo test` exécute les blocs de code de la doc), modules, et une API publique dont les variantes d'erreur sont un engagement.
