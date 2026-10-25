# Labo 07 : Formes et sorties

**Chapitre** : [7. Traits et génériques](../../cours/07-traits-et-generiques.md)

## Objectif
Écrire un trait `Forme` avec une méthode par défaut, l'implémenter pour trois types, et manipuler ces formes de deux façons : en générique (`aire_totale<T: Forme>`, un seul type à la fois, compilé pour chacun) et en objets trait (`Vec<Box<dyn Forme>>`, des types mélangés, un appel par table virtuelle). Puis séparer un calcul de l'endroit où il s'écrit avec un trait `Sortie`, implémenté par la console et par un tampon en mémoire qui rend le rapport testable. Et brancher ton propre type sur les boucles `for` et les chaînes d'itérateurs en implémentant `Iterator`. À la fin, tu sais quand écrire `<T: Trait>`, quand écrire `dyn Trait`, et ce que `From`, `Display` et `Iterator` te donnent gratuitement.

## Consignes
1. Lis `src/lib.rs` : les traits, les types et les signatures sont écrits, chaque corps est un `todo!("TODO n")`. Lis `src/main.rs`, qui construit des formes et appelle tout. Lance `cargo test` : treize tests sur quatorze échouent (le quatorzième ne vérifie que la compilation), mais ça compile.
2. `impl Forme for Cercle` (TODO 1) et `impl Forme for Rectangle` (TODO 2) : trois méthodes chacun. Les tests comparent les flottants à un millionième près, jamais avec `==`. `cargo test cercle` puis `cargo test rectangle` (ce dernier échouera encore sur `From`, c'est le TODO 7).
3. `Triangle::new(a, b, c) -> Option<Triangle>` (TODO 3) et `impl Forme for Triangle` (TODO 4) : la formule de Héron donne l'aire à partir des seuls côtés. `new` refuse un côté nul, négatif ou `NaN`, et un côté plus long que la somme des deux autres. Quand c'est fait, retire le `#[allow(dead_code)]` au-dessus de `Triangle`. `cargo test triangle`.
4. `decrire`, la méthode par défaut du trait (TODO 5) : aucun des trois types ne la redéfinit, et elle marche pour les trois. `cargo test decrire`.
5. `Display` pour les trois types (TODO 6) et `From<(f64, f64)>` pour `Rectangle` (TODO 7) : regarde le test `rectangle_aire_perimetre_et_from`, qui utilise `into()` sans que tu l'aies écrit. `cargo test affichage`.
6. `plus_grande` (TODO 8), `aire_totale` et `aire_totale_dyn` (TODO 9), `trier_par_aire` (TODO 10) : les flottants n'étant pas `Ord`, `max_by` et `sort_by` avec `total_cmp`. Compare les deux `aire_totale` : le corps est identique, seule la signature change, et c'est elle qui décide comment Rust compile l'appel. Essaie d'appeler `aire_totale(&melange())` : lis l'erreur, elle dit que `Box<dyn Forme>` n'implémente pas `Forme`. `cargo test aire_totale`.
7. `Sortie` pour `Console` et `Tampon` (TODO 11), puis `rapport` (TODO 12) : le test passe un `Tampon` et compare les lignes une à une ; `main` passe un `Console`. Le même code, deux destinations, aucune capture de la sortie standard. `cargo test rapport`.
8. `Iterator` pour `Compteur` (TODO 13) : une seule méthode, `next`. Les tests utilisent ensuite `collect`, `map`, `filter`, `sum` et `count`, que tu n'as pas écrits. `cargo test compteur`.
9. `cargo run` : compare avec la sortie attendue. Puis `cargo test`, `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test triangle              # seulement les tests dont le nom contient triangle
cargo run                        # ton programme
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run
formes construites : 4
1. rectangle : aire 12.00, périmètre 14.00
2. cercle : aire 7.07, périmètre 9.42
3. triangle : aire 6.00, périmètre 12.00
4. rectangle : aire 1.00, périmètre 4.00
total : 4 formes, aire 26.07
plus grande : rectangle
par aire croissante : rectangle 1.0 < triangle 6.0 < cercle 7.1 < rectangle 12.0
  rectangle 4 x 3
  rectangle 5 x 2.5
  rectangle 2 x 2
surface des pièces : 28.50 m²
somme des carrés de 1 à 10 : 385
```

## Pour aller plus loin
- Ajoute un `Carre(f64)` qui implémente `Forme` et redéfinit `decrire` pour dire « carré de côté 2 ». Aucune autre ligne du programme ne change : `rapport`, `plus_grande` et `trier_par_aire` le prennent tel quel. C'est l'ensemble ouvert de la section 7.4.
- Fais de `Display` un **supertrait** : `pub trait Forme: fmt::Display { ... }`. Chaque forme doit alors être affichable, et `rapport` peut écrire `{forme}` derrière un `dyn Forme`. Qu'est-ce que le compilateur exige de `Cercle`, `Rectangle` et `Triangle` ?
- Écris `struct Fichier(std::fs::File)` qui implémente `Sortie` avec `writeln!` : il faudra `use std::io::Write`, et `writeln!` rend un `Result` dont tu ne sais pas encore quoi faire proprement. Le chapitre 8 répond : `fn ecrire(&mut self, texte: &str) -> io::Result<()>`, et `?`.
