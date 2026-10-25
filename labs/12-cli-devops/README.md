# Labo 12 : CLI DevOps

**Chapitre** : [12. Fichiers, JSON, ligne de commande DevOps](../../cours/12-fichiers-json-cli.md)

## Objectif
Écrire `dsk`, un petit couteau suisse pour le quotidien d'un sysadmin, avec quatre sous-commandes : `gros` (les plus gros fichiers d'un dossier, récursivement), `logs` (filtrer un journal par niveau et compter), `env` (les variables d'environnement d'un préfixe) et `run` (lancer une commande, la chronométrer, capturer ses sorties et rendre un rapport JSON). Chaque sous-commande sait sortir du texte pour un humain ou du JSON pour `jq`. Tu manipules `std::fs` et le parcours récursif, `BufRead`, `std::process::Command`, `anyhow` et son contexte, `serde` et ses attributs, et `clap` en derive avec l'aide générée et les erreurs d'usage en code 2. Toute la logique est dans `src/lib.rs`, testée sans lancer de processus, dans des dossiers temporaires créés et nettoyés par les tests.

Crates : `clap` (derive et env), `serde` (derive), `serde_json`, `anyhow`, celles que le chapitre présente. Pas de `tempfile` : les tests en écrivent l'équivalent en vingt lignes avec `std::env::temp_dir()` et un `Drop`.

## Consignes
1. Lis `src/lib.rs` (les types sont donnés, les fonctions sont des `todo!`), `src/main.rs` (la CLI clap, presque complète) et `tests/dsk.rs`, en particulier `DossierTemp` : un dossier unique sous `std::env::temp_dir()` que son `Drop` supprime, même quand le test échoue. Lance `cargo test --no-fail-fast` : ça compile, presque tout échoue.
2. `taille_lisible` (TODO 1) : `"0 o"`, `"1.5 Kio"`, `"3.0 Gio"`. `cargo test taille`.
3. `parcourir` (TODO 3) puis `plus_gros` (TODO 2) : `fs::read_dir` en récursif, `file_type()` qui ne suit pas les liens, le tri décroissant avec départage par chemin, et `with_context` pour que l'erreur dise quel dossier manque. Le test `plus_gros_dossier_absent_dit_lequel` vérifie le message **et** que la cause `io::ErrorKind::NotFound` est toujours là, retrouvable par `downcast_ref`. `cargo test plus_gros`.
4. `parser_ligne` (TODO 4), `parser_logs` (TODO 5) et `bilan` (TODO 6) : `splitn(4, ' ')`, `?` sur des `Option`, une chaîne d'itérateurs sur `lines()`. Le test lit le même journal depuis un `&[u8]` et depuis un vrai fichier : c'est l'intérêt de `impl BufRead`. `cargo test logs`.
5. L'attribut serde du `Niveau` (TODO 9) : sans lui, le JSON dit `"Error"`. `cargo test json`.
6. `filtrer_variables` (TODO 7). Note pourquoi elle existe à côté de `variables` : modifier l'environnement d'un processus de test est `unsafe` depuis l'édition 2024 (`std::env::set_var`), parce que d'autres threads le lisent peut-être au même moment. On teste donc le filtre avec une fausse liste, et `variables` sur les `CARGO_PKG_*` que `cargo test` définit déjà. `cargo test variables`.
7. `lancer` (TODO 8) : `Command::new(...).args(...).output()`, le code de sortie, la durée, les deux sorties. Une commande introuvable est une erreur (`with_context` nomme la commande), une commande qui rend 3 n'en est pas une : c'est un rapport avec `code: 3`. `cargo test lancer`.
8. Dans `src/main.rs`, le TODO 10 : rends `--prefixe` facultatif avec `default_value` et `env`. Les tests de `main.rs` vérifient la CLI elle-même avec `Cli::try_parse_from`, sans lancer de processus, y compris le code 2 des erreurs d'usage. Puis `cargo run -- env --help` : l'aide affiche `[env: DSK_PREFIXE=] [default: DSK_]` toute seule.
9. `cargo test` : tout vert. `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` : rien à redire. Essaie l'outil pour de vrai : `cargo run -q -- gros ~ --top 5`, `cargo run -q -- gros . --json | jq '.[0]'`, `cargo run -q -- run -- ls -la`.

## Comment lancer
```bash
cargo test                          # tes tests (rouge tant que les TODO restent)
cargo test --no-fail-fast           # tous les binaires de test, même après un échec
cargo test plus_gros                # les tests dont le nom contient plus_gros
cargo run -- --help                 # l'aide générée par clap
cargo run -q -- gros . --top 5
cargo run -q -- logs app.log --niveau warn --json
DSK_ROLE=web cargo run -q -- env
cargo run -q -- run -- ls -d /tmp /nulle/part ; echo "code $?"
(cd solution && cargo test)         # la solution (vert)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
Dans un dossier de démonstration avec une sauvegarde de 3,5 Mo et deux journaux :
```
$ dsk gros . --top 3
   3.3 Mio  ./srv/backup/base.dump
   1.1 Mio  ./var/log/app.log.1
   2.0 Kio  ./var/log/app.log
3 fichier(s) sous .
$ dsk gros . --top 1 --json
[
  {
    "chemin": "./srv/backup/base.dump",
    "taille": 3500000
  }
]
1 fichier(s) sous .
$ dsk logs app.log --niveau warn
2026-10-25 01:02:04 WARN  web1 lent (1832 ms)
2026-10-25 01:02:05 ERROR db1:5432 connexion refusée
2026-10-25 01:02:06 ERROR disque plein sur /var
3 entrée(s) : WARN 1, ERROR 2
$ DSK_REGION=eu-west DSK_ROLE=web dsk env
DSK_REGION=eu-west
DSK_ROLE=web
$ DSK_PREFIXE=HOM dsk env --json
{
  "HOME": "/Users/stranix"
}
$ dsk run -- ls -d /tmp /nulle/part ; echo "code $?"
{
  "commande": "ls -d /tmp /nulle/part",
  "code": 1,
  "duree_ms": 8,
  "stdout": "/tmp\n",
  "stderr": "ls: /nulle/part: No such file or directory\n"
}
code 1
$ dsk logs app.log --niveau PANIQUE ; echo "code $?"
error: invalid value 'PANIQUE' for '--niveau <NIVEAU>': niveau "PANIQUE" inconnu (DEBUG, INFO, WARN ou ERROR)

For more information, try '--help'.
code 2
$ dsk logs absent.log ; echo "code $?"
dsk : ouvrir absent.log: No such file or directory (os error 2)
code 1
```
Les lignes `N fichier(s)` et `N entrée(s)` vont sur stderr : `dsk gros . --json | jq` ne reçoit que le JSON. Les messages de clap sont en anglais, ceux de `dsk` en français : clap n'est pas traduit, et c'est très bien comme ça pour un outil qu'on lit dans des journaux de CI.

## Pour aller plus loin
- Ajoute à `dsk logs` une option `--suivre` qui lit le fichier en continu, comme `tail -f` : une boucle qui relit `BufRead::read_line` et dort 200 ms quand elle rend 0 octet. Puis gère la rotation (le fichier remplacé par un neuf) en comparant `metadata().len()` à ta position.
- Ajoute une sous-commande `dsk config FICHIER.toml` qui lit une configuration avec la crate `toml` et les mêmes `#[derive(Deserialize)]` : c'est le même serde, seul le format change. Ajoute `#[serde(deny_unknown_fields)]` et tape une faute dans une clé : l'erreur dit laquelle et à quelle ligne.
- Transforme `dsk run` en petit superviseur : relance la commande quand elle échoue, au plus N fois, avec une attente qui double à chaque essai, et un rapport JSON par tentative. Puis expose le dernier rapport en HTTP avec axum : c'est le chapitre 13.
