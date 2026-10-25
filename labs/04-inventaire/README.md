# Labo 04 : Inventaire

**Chapitre** : [4. Possession et emprunts : le cœur de Rust](../../cours/04-possession-et-emprunts.md)

## Objectif
Gérer un petit stock d'articles, un `Vec<Article>` où chaque article possède son nom (`String`), sa quantité et son prix en centimes. Ajouter, chercher, retirer, compter, fusionner, afficher. Le code de chaque fonction est court ; ce qui compte, ce sont les signatures. Chacune dit qui possède quoi : `&[Article]` pour lire, `&mut Vec<Article>` pour modifier, `Vec<Article>` pour prendre, `<'a>` quand le compilateur ne peut pas deviner d'où vient une référence. À la fin tu sais lire une signature Rust comme un contrat, et tu as vu en vrai les erreurs `E0382`, `E0499`, `E0502` et `E0106`.

## Consignes
1. Lis `src/lib.rs` en entier, signatures et commentaires d'abord : ils expliquent pourquoi chaque paramètre est un emprunt, un emprunt mutable ou une possession. Lis `tests/inventaire.rs`, en particulier `chercher_survit_au_nom` et `ajouter_accepte_une_string_empruntee`. `cargo test` : ça compile, tout échoue.
2. `ajouter(inv: &mut Vec<Article>, nom: &str, quantite, prix_centimes)` : fusionne la quantité si le nom existe (le prix d'origine reste), sinon pousse un nouvel article. `inv.iter_mut().find(...)` donne une `&mut Article` modifiable sur place. `cargo test ajouter`.
3. `chercher<'a>(inv: &'a [Article], nom: &str) -> Option<&'a Article>` : une ligne. Puis retire les deux `'a` de la signature, relance `cargo build`, et lis l'erreur `E0106` : le compilateur te demande à laquelle des deux entrées le résultat emprunte. Remets-les. `cargo test chercher`.
4. `retirer(inv: &mut Vec<Article>, nom: &str, quantite: u32) -> Result<u32, String>` : rend la quantité restante, supprime l'article qui tombe à zéro, et rend `Err` (en citant le nom) si l'article est inconnu ou le stock insuffisant, sans rien modifier. Travaille avec un indice (`position`), pas avec une référence : le commentaire du `TODO 3` dit pourquoi. Si tu veux voir `E0499` de tes yeux, essaie d'abord avec `iter_mut().find(...)` puis `inv.remove(...)`. `cargo test retirer`.
5. `valeur_totale(inv: &[Article]) -> u64` (en `u64`, le test `sans_debordement_u32` y veille), `plus_cher(inv: &[Article]) -> Option<&Article>` et `noms(inv: &[Article]) -> Vec<&str>`. Ces deux dernières n'écrivent aucune durée de vie : la règle d'élision s'en charge, la solution explique laquelle. `cargo test valeur`, `cargo test plus_cher`, `cargo test noms`.
6. `fusionner(a: Vec<Article>, b: Vec<Article>) -> Vec<Article>` : prend possession des deux inventaires et déplace les articles de `b` dans `a`, sans un seul `.clone()`. `cargo test fusionner`.
7. `format_tableau(inv: &[Article]) -> String` : le format exact est dans la documentation de la fonction et dans le test `format_tableau_exact`. `cargo test format`.
8. Dans `src/main.rs`, après `let tout = fusionner(stock, arrivage);`, ajoute `println!("{}", stock.len());` et lis `E0382`. Puis, avant l'affichage de `noms(&stock)`, garde le résultat dans une variable, appelle `ajouter(&mut stock, ...)` entre les deux, et lis `E0502`. Retire tes essais.
9. Retire les deux `#![allow(...)]` en haut de `src/lib.rs`, puis `cargo test`, `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test retirer               # un seul groupe de tests
cargo run                        # la démo
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run
Article       Qté      Prix     Total
clavier         3     49.90    149.70
souris          5     19.90     99.50
écran           1    189.00    189.00
TOTAL                          438.20

souris : 5 en stock
retrait de 2 souris, reste 3
erreur : stock insuffisant pour écran : 1 en stock, 3 demandés
erreur : article inconnu : webcam
le plus cher : écran (18900 centimes)
noms : ["clavier", "souris", "écran"]

Article       Qté      Prix     Total
clavier         3     49.90    149.70
souris         13     19.90    258.70
écran           1    189.00    189.00
câble           4      5.90     23.60
TOTAL                          621.00
```
Les messages d'erreur de ta version peuvent différer : les tests vérifient seulement qu'ils citent le nom de l'article.

## Pour aller plus loin
- Écris `chercher_mut<'a>(inv: &'a mut [Article], nom: &str) -> Option<&'a mut Article>`, puis réécris `ajouter` avec. Essaie la forme `if let Some(a) = chercher_mut(inv, nom) { ... return; } inv.push(...)` : elle compile. Essaie ensuite de rendre la référence trouvée depuis la fonction appelante ou de la garder au-delà du `push` : tu touches une limite connue du vérificateur actuel, et la réponse est encore un indice.
- Remplace `Vec<Article>` par une `HashMap<String, Article>` indexée par nom : `ajouter` devient trois lignes avec l'API `entry`, mais l'ordre d'insertion est perdu. Le chapitre 5 fait le tour des collections et des itérateurs qui remplacent les boucles de ce labo.
- `noms` rend un `Vec<&str>` qui bloque l'inventaire tant qu'il vit. Écris une version `noms_possedes(inv: &[Article]) -> Vec<String>` et compare : quand l'appelant a-t-il besoin de l'une, quand de l'autre ?
