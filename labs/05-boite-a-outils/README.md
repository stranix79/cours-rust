# Labo 05 : Boîte à outils

**Chapitre** : [5. Collections et itérateurs](../../cours/05-collections-et-iterateurs.md)

## Objectif
Écrire huit petites fonctions de sysadmin (compter les mots d'un journal, les plus fréquents, dédoublonner, des statistiques de latence, grouper des fichiers par extension, analyser et compter des lignes de log) en manipulant `Vec`, `HashMap`, `BTreeMap`, `HashSet` et des chaînes d'itérateurs. Chaque fonction tient en une ou deux chaînes : si tu écris une boucle avec trois `if` et un compteur, cherche l'adaptateur qui le fait. À la fin, tu lis et tu écris `iter().filter().map().collect()` sans y penser.

## Consignes
1. Lis `src/lib.rs` : les types `Stats`, `Niveau` et `LigneLog` sont écrits, les huit fonctions ont un corps `todo!("TODO n")`. Lis `src/main.rs`, qui appelle tout sur `exemple.log` (compilé dans le binaire avec `include_str!`). Lance `cargo test` : tout échoue sauf `niveau_est_ordonne`, mais ça compile.
2. `compter_mots(texte) -> HashMap<String, usize>` (TODO 1) : en minuscules, un mot est une suite de caractères alphanumériques. Les tests vérifient les accents (`liège`, `été`), l'apostrophe (`l'été` donne `l` et `été`), le tiret (`web-01` donne `web` et `01`) et le texte vide. `cargo test compter_mots`.
3. `top_n(&compte, n) -> Vec<(String, usize)>` (TODO 2) : nombre décroissant, puis mot croissant à égalité. Le test lance vingt fois le même calcul : sans le second critère, l'ordre aléatoire de la `HashMap` le ferait échouer de temps en temps. `cargo test top_n`.
4. `dedupliquer(elements) -> Vec<String>` (TODO 3) : la première occurrence gagne, l'ordre est conservé. `sort` puis `dedup` ne convient pas, regarde pourquoi dans le chapitre (section 5.2). `cargo test dedupliquer`.
5. `statistiques(&valeurs) -> Option<Stats>` (TODO 4) : `None` pour une tranche vide ; médiane d'un nombre pair de valeurs = moyenne des deux du milieu. Un test vérifie que tu ne tries pas la tranche de l'appelant (tu ne peux pas, elle est en `&[f64]`, mais ça vaut le coup de le voir). `cargo test statistiques`.
6. `grouper_par_extension(&chemins) -> BTreeMap<String, Vec<String>>` (TODO 5) : `Path::extension` fait le travail difficile (`.bashrc` n'a pas d'extension, `db.sql.gz` a `gz`). `cargo test grouper`.
7. `niveau_depuis(texte) -> Option<Niveau>` (TODO 6) et `parser_ligne_log(ligne) -> Option<LigneLog>` (TODO 7) : le format est `2026-10-25 01:02:03 ERROR message avec espaces`, et le message garde ses espaces intérieurs. `splitn(4, ' ')` et quatre `next()?` suffisent. `cargo test parser`.
8. `compter_par_niveau(&lignes) -> BTreeMap<Niveau, usize>` (TODO 8) : le test vérifie aussi l'ordre des clés, `Debug` avant `Info` avant `Error`, que tu obtiens gratuitement grâce au `derive(Ord)` de `Niveau` et à la `BTreeMap`. `cargo test compter_par_niveau`.
9. `cargo run` : compare avec la sortie attendue ci-dessous. Puis `cargo test`, `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test top_n                 # seulement les tests dont le nom contient top_n
cargo run                        # ton programme, sur le journal intégré
cargo run -- exemple.log         # le même, lu depuis le fichier
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run
lignes lues : 12, illisibles : 1
  Debug  1
  Info   4
  Warn   3
  Error  3
mots les plus fréquents :
  01         7
  db         5
  latence    3
erreurs distinctes :
  connexion refusée par db-01
  disque plein sur /var/lib/postgresql
latences : min 40 max 180 moyenne 105.0 médiane 95
fichiers par extension :
  (aucune) /var/lib/postgresql
  gz       /var/backups/db.sql.gz
  log      /var/log/app.log
  toml     /etc/app/config.toml
$ cargo run -- /nulle/part.log
impossible de lire /nulle/part.log : No such file or directory (os error 2)
```

## Pour aller plus loin
- Le mot le plus fréquent est `01`, parce que le tiret coupe `web-01` en deux. Écris `compter_mots_hotes` qui garde les tirets à l'intérieur d'un mot (`c.is_alphanumeric() || c == '-'`), puis retire les tirets en bout de mot avec `trim_matches('-')`. Que devient le classement ?
- Lance `cargo run -- /var/log/system.log` (ou un journal de ton serveur) : combien de lignes sont illisibles pour notre format ? Ajoute un `--niveau ERROR` qui filtre l'affichage, en lisant `env::args()` avec `skip(1)` et un `match`.
- `parser_ligne_log` rend `None` sans dire pourquoi. Le chapitre 6 montre comment `Option` se transforme en `Result` avec `ok_or`, et le chapitre 8 comment faire une vraie erreur qui dit « ligne 7 : niveau inconnu FATAL ».
