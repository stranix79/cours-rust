# cours-rust : Rust de zéro à la prod, pour qui a déjà codé

```
$ whoami
stranix · le renard qui aime savoir ce qui se passe sous le capot

$ cat cours-rust/README.md
```

Un cours en français, par **Gilles Fauvie** ([stranix.net](https://stranix.net)).

Tu as déjà codé. Du C, du Python, peut-être du Go. Tu fais tourner des serveurs, des bases de données, des conteneurs. Et tu as remarqué que les outils qui ont remplacé tes vieux réflexes ces dernières années, `ripgrep` à la place de `grep`, `fd`, `bat`, `uv` et `ruff` côté Python, Alacritty, Zed, le moteur de Deno, Firecracker sous les micro-VM, sont écrits dans le même langage. Et que le noyau Linux a fini par l'accepter là où il refusait tout sauf le C. Ce n'est pas un hasard, et ce n'est pas pour les mêmes raisons que Go.

Ce dépôt est un cours **qui part de zéro en Rust** mais pas de zéro en programmation. Chaque notion est comparée à ce que tu connais (C, Python, et Go si tu as suivi [cours-go](https://github.com/stranix79/cours-go)), et chaque chapitre répond d'abord à la question que tu te poses vraiment : *pourquoi c'est comme ça, et dans quel cas je m'en sers ?* Le chapitre 0 est entièrement consacré à « quand utiliser Rust, et quand Go ou Python suffisent ». Le chapitre 4, le plus important du cours, est consacré à la possession : ce qui fait que Rust est sûr sans ramasse-miettes, et ce qui fait râler tout débutant pendant une semaine.

Tout a été écrit et exécuté avec **Rust 1.93** (édition 2024) sur un Mac Apple Silicon, mais rien n'est spécifique à macOS : Linux et Windows font aussi bien, et les binaires que tu produiras tourneront sur les trois.

## Comment ça marche

Dix-sept chapitres, seize labos. Le labo donne un squelette à compléter avec des `// TODO`, une solution commentée ligne par ligne dans `solution/`, et, à partir du labo 02, des tests `cargo test` qui te disent si c'est bon. Compte une à deux heures par chapitre, un peu plus pour le chapitre 4.

| Chapitre | Labo |
|---|---|
| [0. Pourquoi Rust, et quand l'utiliser (ou pas)](cours/00-pourquoi-rust.md) | aucun |
| [1. Installer, outiller, lancer](cours/01-installer-et-outiller.md) | [01-hello](labs/01-hello/README.md) |
| [2. Variables, types, chaînes](cours/02-variables-types-chaines.md) | [02-convertisseur](labs/02-convertisseur/README.md) |
| [3. Contrôle : if, loop, while, for, match](cours/03-controle.md) | [03-devine-nombre](labs/03-devine-nombre/README.md) |
| [4. Possession et emprunts : le cœur de Rust](cours/04-possession-et-emprunts.md) | [04-inventaire](labs/04-inventaire/README.md) |
| [5. Collections et itérateurs](cours/05-collections-et-iterateurs.md) | [05-boite-a-outils](labs/05-boite-a-outils/README.md) |
| [6. Structs, enums, méthodes, Option](cours/06-structs-enums-option.md) | [06-compte-bancaire](labs/06-compte-bancaire/README.md) |
| [7. Traits et génériques](cours/07-traits-et-generiques.md) | [07-formes-et-io](labs/07-formes-et-io/README.md) |
| [8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic](cours/08-erreurs.md) | [08-robuste](labs/08-robuste/README.md) |
| [9. Modules, crates, Cargo, tests, documentation](cours/09-modules-crates-tests.md) | [09-ma-crate](labs/09-ma-crate/README.md) |
| [10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe](cours/10-sous-le-capot.md) | [10-sous-le-capot](labs/10-sous-le-capot/README.md) |
| [11. Concurrence : threads, channels, Arc et Mutex, Send et Sync](cours/11-concurrence.md) | [11-pinger-parallele](labs/11-pinger-parallele/README.md) |
| [12. Fichiers, JSON, ligne de commande DevOps](cours/12-fichiers-json-cli.md) | [12-cli-devops](labs/12-cli-devops/README.md) |
| [13. HTTP et async : tokio, reqwest, axum](cours/13-http-et-async.md) | [13-api-todo](labs/13-api-todo/README.md) |
| [14. Bases de données : SQLite et PostgreSQL](cours/14-bases-de-donnees.md) | [14-sqlite-postgres](labs/14-sqlite-postgres/README.md) |
| [15. Qualité, build, release, Docker](cours/15-qualite-build-release.md) | [15-release](labs/15-release/README.md) |
| [16. Projet final et pour aller plus loin](cours/16-projet-final-et-suite.md) | [16-projet](labs/16-projet/README.md) |

Antisèches : [tools/rust-antiseche.md](tools/rust-antiseche.md) (la syntaxe sur deux pages) et [tools/c-python-go-vs-rust.md](tools/c-python-go-vs-rust.md) (la table de correspondance C / Python / Go / Rust).

## Démarrer en deux minutes

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # si Rust n'est pas installé
git clone https://github.com/stranix79/cours-rust.git
cd cours-rust
cargo --version                 # 1.85 minimum, 1.93 recommandé
cd labs/01-hello
cargo run                       # ton premier programme
(cd solution && cargo run)      # la version corrigée, commentée
```

À partir du labo 02 : `cargo test` dans le dossier du labo te dit si ta version passe, et `(cd solution && cargo test)` lance les mêmes tests sur la solution. Les labos partagent un espace de travail Cargo (le `Cargo.toml` à la racine) : les dépendances ne se compilent qu'une fois, dans un seul dossier `target/`. Pas de `venv`, pas d'installation globale : `cargo` suffit.

## Ce que tu sauras faire à la fin

- dire, en une phrase, si un problème donné mérite du Rust, du Go, du Python ou du shell ;
- lire couramment le code des outils que tu utilises (ripgrep, uv, ruff) et le code Rust qu'une IA t'écrit, et voir ce qui cloche ;
- écrire du Rust idiomatique, pas du C ni du Go traduit mot à mot : possession, emprunts, `Option`, `Result`, itérateurs, traits ;
- comprendre ce que le vérificateur d'emprunts te reproche, et pourquoi il a raison ;
- gérer les erreurs sans exceptions et sans `unwrap()` partout ;
- structurer un projet en modules et en crates, avec des tests, de la documentation, `clippy` et `rustfmt` ;
- écrire des programmes parallèles que le compilateur garantit sans course de données ;
- écrire des outils en ligne de commande pour ton quotidien DevOps : fichiers, JSON, processus, options ;
- écrire un client HTTP, une API JSON asynchrone, parler à SQLite et à PostgreSQL ;
- produire un binaire statique pour Linux, macOS et Windows, et l'emballer dans une image Docker de quelques méga-octets ;
- comprendre ce qui se passe sous le capot : pile, tas, durées de vie, abstractions à coût nul, et ce que `unsafe` veut vraiment dire.

## Pourquoi en français, pourquoi comme ça

Parce que les cours Rust en français s'arrêtent à `println!`, et que le livre officiel, excellent, suppose que tu as le temps de le lire en entier. Ici, on commence par le pourquoi, on compare à ce que tu connais, et on va jusqu'au bout : un service qui tourne en production, dans un conteneur, avec ses tests.

Licence MIT. Le cours est vivant : une erreur, une question, une idée de labo, ouvre une issue. On en parle aussi sur [stranix.net](https://stranix.net).

## Du même auteur

Les autres cours, même esprit :

- [cours-go](https://github.com/stranix79/cours-go) : Go de zéro à la prod, même format, même public. Le bon complément : ce cours Rust y renvoie souvent.
- [cours-python](https://github.com/stranix79/cours-python) : Python de zéro à FastAPI.
- [asm-m1](https://github.com/stranix79/asm-m1) : l'assembleur ARM64 sur Mac Apple Silicon, pour voir ce qu'il y a sous le capot.

Et le reste :

- [stranix.net](https://stranix.net) : le blog, où l'on parle de ce genre de choses.
- [Deckhand](https://deckhand.show) : des présentations HTML écrites en Markdown.
- [Chutag](https://chutag.stranix.net) : des tags QR pour retrouver ses objets perdus (bêta).
- [apps.stranix.net](https://apps.stranix.net) : les apps macOS de Stranix.
- [CODE79](https://code79.com) : l'atelier devops derrière tout ça.

```
$ ls /ventures
asm-m1/    cours-python/    cours-go/    cours-rust/    deckhand/
```
