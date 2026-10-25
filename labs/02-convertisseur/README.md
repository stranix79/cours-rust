# Labo 02 : Convertisseur

**Chapitre** : [2. Variables, types, chaînes](../../cours/02-variables-types-chaines.md)

## Objectif
Écrire une petite bibliothèque de conversions de sysadmin : des températures, une taille en octets rendue lisible (`1536` donne `1.5 Ko`), et une durée écrite à la main (`1h30m`) transformée en `std::time::Duration`. Tu manipules des `f64`, des `u64`, les conversions explicites (`as`, `u64::from`), `format!` avec largeur et précision, une `String` qu'on remplit caractère par caractère, et ton premier `Result`. C'est aussi le premier labo avec des tests : `cargo test` te dit si c'est juste.

## Consignes
1. Lis `Cargo.toml` : il déclare une bibliothèque (`[lib] name = "convertisseur"`, dans `src/lib.rs`) et un binaire (`convertisseur`, dans `src/main.rs`) dans le même paquet. Lis ensuite `src/lib.rs` et `tests/convertisseur.rs`. Lance `cargo test` : tout compile, et tous les tests échouent sur un `todo!()`.
2. `fahrenheit(celsius: f64) -> f64` et `celsius(fahrenheit: f64) -> f64` : une ligne chacune. Écris `9.0`, pas `9` : un `f64` ne se multiplie pas par un `i32`. `cargo test fahrenheit` puis `cargo test celsius` (le second lance aussi l'aller-retour).
3. `taille_lisible(octets: u64) -> String` : `"512 o"` sous 1024, sans décimale ; au-delà, divise par 1024 en passant par `Ko`, `Mo`, `Go`, `To`, et affiche une décimale (`{:.1}`). Au-delà du To, on reste en To : `5000.0 To`. Les tests vont jusqu'à `u64::MAX`. `cargo test taille`.
4. `parse_duree(texte: &str) -> Result<Duration, String>` : la bibliothèque standard n'a pas d'analyseur de durées (Go a `time.ParseDuration`, Rust non), on l'écrit. Une suite de nombres entiers, chacun suivi d'une unité parmi `h`, `m`, `s`, `ms` : `"1h30m"`, `"45s"`, `"250ms"`, `"1h1m1s1ms"`. Chaîne vide, unité inconnue (`"10x"`), nombre sans unité (`"45"`), unité sans nombre (`"h"`), décimale ou espace : `Err`, avec le texte complet dans le message. Le commentaire du `TODO 4` découpe le travail. `cargo test duree`.
5. `secondes(d: Duration) -> u64` : le nombre de secondes entières, tronqué. Une méthode de `Duration` fait exactement ça : trouve-la dans la documentation (`cargo doc --open`, ou docs.rs). `cargo test secondes`.
6. Retire la ligne `#![allow(unused_variables)]` en haut de `src/lib.rs`, puis `cargo test`, `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire. Lance `cargo run` et compare avec la sortie attendue.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test taille                # seulement les tests dont le nom contient « taille »
cargo run                        # le tableau de démonstration
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run
Celsius     Fahrenheit
-40.0            -40.0
0.0               32.0
20.0              68.0
37.0              98.6
100.0            212.0
98.6 °F           37.0

Octets               Lisible
512                    512 o
1536                  1.5 Ko
10485760             10.0 Mo
2684354560            2.5 Go
3298534883328         3.0 To
5497558138880000   5000.0 To

Durée         Secondes
"1h30m"           5400
"45s"               45
"250ms"              0
"2h"              7200
"1h1m1s1ms"       3661
"10x"           erreur  (unité inconnue "x" dans "10x")
""              erreur  (durée vide : "")
```
Les messages d'erreur de ta version peuvent différer : les tests vérifient seulement qu'ils contiennent le texte fautif.

## Pour aller plus loin
- `taille_lisible(1_048_575)` rend `"1024.0 Ko"` : la valeur est sous 1024 avant l'arrondi, au-dessus après. Corrige le cas (arrondis d'abord, compare ensuite) et ajoute le test.
- Ajoute l'unité `d` (jours) à `parse_duree`, puis une fonction `format_duree(d: Duration) -> String` qui fait le chemin inverse : `5400 s` donne `"1h30m"`. Un test d'aller-retour sur une dizaine de valeurs vaut mieux que dix tests à la main.
- `parse_duree` rend une `String` comme erreur : c'est pratique et pauvre, l'appelant ne peut pas distinguer « vide » de « unité inconnue » sans lire le texte. Le chapitre 8 en fera un `enum` d'erreurs avec `thiserror`, et le chapitre 3 te fait déjà écrire ton premier `enum` dans le [labo 03](../03-devine-nombre/README.md).
