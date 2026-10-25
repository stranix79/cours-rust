# Labo 03 : Devine le nombre

**Chapitre** : [3. Contrôle : if, loop, while, for, match](../../cours/03-controle.md)

## Objectif
Le jeu classique : le programme tire un nombre entre 1 et 100, tu proposes, il répond « plus grand » ou « plus petit » jusqu'à ce que tu trouves. La logique (juger un essai, lire une ligne, rejouer une partie, choisir le message) est dans `src/lib.rs`, testée sans clavier ni hasard ; `src/main.rs` tire le secret avec la crate `rand` et boucle sur l'entrée standard. Tu y écris des `match` exhaustifs, un motif `n @ 1..=100`, une garde, un `let else`, une boucle `loop` avec `break` et `continue`, et tu ajoutes ta première dépendance.

## Consignes
1. Lis `Cargo.toml` : une dépendance, `rand = "0.9"`. Au premier `cargo build`, Cargo la télécharge depuis crates.io, la compile, et note la version exacte dans le `Cargo.lock` à la racine du dépôt. Lis `src/lib.rs` (l'`enum Verdict` et la `struct Resultat` sont fournis) et `tests/devine.rs`. `cargo test` : ça compile, tout échoue.
2. `evaluer(secret, essai) -> Verdict` : `TropPetit` si l'essai est sous le secret, `TropGrand` au-dessus, `Trouve` sinon. Essaie le `match essai.cmp(&secret)`, et retire un des trois bras pour lire l'erreur `E0004`. `cargo test evaluer`.
3. `lire_essai(texte: &str) -> Option<u32>` : `trim`, `parse::<u32>()`, et `None` pour tout ce qui n'est pas un entier de 1 à 100 (vide, lettres, négatif, décimal, 0, 101, trop grand pour un `u32`). Un seul `match` suffit. `cargo test lire_essai`.
4. `partie(secret, essais: &[&str]) -> Resultat` : rejoue une liste de lignes comme si le joueur les tapait. Les lignes invalides vont dans `invalides` et ne comptent pas comme tentatives ; la partie s'arrête au premier essai gagnant, les lignes suivantes sont ignorées. `cargo test partie`.
5. `message(verdict, tentatives) -> String` : `"C'est plus grand."`, `"C'est plus petit."`, `"Trouvé du premier coup !"` pour une seule tentative, `"Trouvé en 7 tentatives."` sinon. Une garde `if` dans le `match`. `cargo test message`.
6. Dans `src/main.rs`, le `TODO 5` tire le secret (`rand::rng().random_range(1..=100)` ; le `use rand::Rng;` est indispensable, sans lui la méthode n'existe pas) et le `TODO 6` écrit la boucle de jeu avec `std::io::stdin().read_line`. Joue une partie avec `cargo run`, puis sans clavier : `printf '50\n25\n' | cargo run` doit s'arrêter proprement en fin d'entrée.
7. Retire les `#![allow(...)]` et `#[allow(...)]` du squelette, puis `cargo test`, `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test lire_essai            # un seul groupe de tests
cargo run                        # jouer au clavier (Ctrl-D pour abandonner)
printf '50\n25\n75\n' | cargo run   # jouer sans clavier
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
Le secret change à chaque partie, la forme non. Les lignes `cinquante`, `50`, `25`, `12` sont celles que le joueur a tapées :
```
$ cargo run
J'ai choisi un nombre entre 1 et 100. À toi.
cinquante
Un nombre entre 1 et 100, s'il te plaît.
50
C'est plus petit.
25
C'est plus petit.
12
Trouvé en 3 tentatives.
```
Et en fin d'entrée sans avoir trouvé :
```
$ printf '50\n' | cargo run
J'ai choisi un nombre entre 1 et 100. À toi.
C'est plus grand.
Abandon. C'était 83.
```

## Pour aller plus loin
- Limite la partie à 7 tentatives (la dichotomie trouve toujours en 7 coups sur 1 à 100 : pourquoi ?). Ajoute un champ `perdu` à `Resultat`, ou mieux, remplace `trouve: bool` par un `enum Issue { Gagne, Perdu, Abandon }` : le chapitre 6 dira pourquoi un enum vaut mieux que deux booléens.
- Écris un joueur automatique : une fonction `strategie(min, max) -> u32` qui joue la dichotomie, et un test qui vérifie, pour les 100 secrets possibles, que `partie` gagne en 7 tentatives au plus. Il te faudra construire les essais au fil de l'eau : c'est un `Vec<String>`, et passer `&[&str]` à partir de lui demande de comprendre les emprunts, ce qui tombe bien : c'est le chapitre 4 et le [labo 04](../04-inventaire/README.md).
- Accepte `q` ou `quitter` pour abandonner, avec un nouveau cas dans un `enum Saisie { Essai(u32), Quitter, Invalide }` que rend `lire_essai`.
