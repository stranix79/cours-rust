# 3. Contrôle : if, loop, while, for, match

*Rust de zéro à la prod : chapitre 3 sur 16.* ← [2. Variables, types, chaînes](02-variables-types-chaines.md) · [Sommaire](../README.md) · [4. Possession et emprunts : le cœur de Rust](04-possession-et-emprunts.md) →

Tu connais `if`, `while` et `for` depuis toujours, et Rust ne les réinvente pas. Il change deux choses, qui changent ta façon d'écrire. La première : presque tout est une **expression**, qui produit une valeur. Un `if`, un `match`, un bloc entre accolades, une boucle `loop` rendent quelque chose, ce qui supprime l'opérateur ternaire, les variables déclarées vides puis remplies dans chaque branche, et une bonne partie des `return`. La seconde : `match`, le `switch` de Rust, **vérifie à la compilation que tu as traité tous les cas**. Oublie une valeur, et ça ne compile pas. Ce chapitre fait le tour des boucles et des branchements, puis présente `match` et ses motifs, `if let`, `let else`, et ta première vraie rencontre avec `Option`. Il se termine par la lecture de l'entrée standard, dont le labo a besoin.

### 3.1 Tout est une expression

Une **instruction** fait quelque chose et ne rend rien : `let x = 5;`, un appel suivi d'un point-virgule. Une **expression** produit une valeur : `5`, `x + 1`, `double(21)`. En C, en Go et en Python, `if` et les blocs sont des instructions. En Rust, ce sont des expressions :

```rust
fn main() {
    let charge = 0.82;
    let etat = if charge > 0.9 {
        "critique"
    } else if charge > 0.7 {
        "élevée"
    } else {
        "normale"
    };
    println!("charge {etat}");

    let port = {
        let base = 5000;
        let decalage = 432;
        base + decalage
    };
    println!("port {port}");

    let n = 7;
    let parite = if n % 2 == 0 { "pair" } else { "impair" };
    println!("{n} est {parite}");
}
```

```
$ cargo run
charge élevée
port 5432
7 est impair
```

La règle qui fait tout marcher : **un bloc vaut sa dernière expression, si elle n'est pas suivie d'un point-virgule**. `base + decalage` sans `;` est la valeur du bloc ; les deux `let` au-dessus sont des instructions, et `base` et `decalage` disparaissent à l'accolade fermante. Le bloc sert à calculer une valeur avec des variables intermédiaires qui ne polluent pas la suite.

Le point-virgule a donc un sens. Il transforme une expression en instruction et jette sa valeur. C'est la même règle qui fait qu'une fonction n'a pas besoin de `return` : la dernière expression de son corps est la valeur rendue. Et c'est l'erreur que tu feras dix fois la première semaine :

```rust
fn double(x: i32) -> i32 {
    x * 2;
}
```

```
$ cargo build
error[E0308]: mismatched types
 --> src/main.rs:1:22
  |
1 | fn double(x: i32) -> i32 {
  |    ------            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     x * 2;
  |          - help: remove this semicolon to return this value
```

Le point-virgule a jeté `x * 2`, le corps ne rend plus rien, c'est-à-dire `()`, le type « unité », qui ne contient qu'une seule valeur et joue le rôle du `void` du C. Le compilateur te dit exactement quel caractère retirer. `return x * 2;` existe aussi, mais on le réserve aux sorties anticipées, au milieu d'une fonction ; en dernière ligne, on écrit l'expression nue.

### 3.2 `if`, et pas d'opérateur ternaire

Le `if` de Rust ressemble à celui de Go : pas de parenthèses autour de la condition, accolades obligatoires même pour une seule ligne. Et la condition doit être un `bool`, sans conversion : `if n { ... }` avec un entier ne compile pas (`expected bool, found integer`). Pas de « zéro est faux » comme en C, pas de « liste vide est fausse » comme en Python : tu écris `if n != 0` ou `if !v.is_empty()`.

Il n'y a pas d'opérateur ternaire `c ? a : b`, parce qu'il ne servirait à rien : `if c { a } else { b }` est déjà une expression, et tient sur une ligne. Deux contraintes viennent avec. Les deux branches doivent avoir le même type :

```rust
let parite = if n % 2 == 0 { "pair" } else { 1 };
```

```
$ cargo build
error[E0308]: `if` and `else` have incompatible types
 --> src/main.rs:3:50
  |
3 |     let parite = if n % 2 == 0 { "pair" } else { 1 };
  |                                  ------          ^ expected `&str`, found integer
  |                                  |
  |                                  expected because of this
```

Et un `if` dont on garde la valeur doit avoir un `else`, sinon que vaudrait-il quand la condition est fausse ?

```rust
let n = if actif { 1 };
```

```
$ cargo build
error[E0317]: `if` may be missing an `else` clause
 --> src/main.rs:3:13
  |
3 |     let n = if actif { 1 };
  |             ^^^^^^^^^^^-^^
  |             |          |
  |             |          found here
  |             expected integer, found `()`
  |
  = note: `if` expressions without `else` evaluate to `()`
```

Un `if` sans `else` vaut `()`. C'est très bien quand on ne garde pas sa valeur, c'est-à-dire dans l'immense majorité des `if` que tu écriras.

**Venant de Python :** `etat = "critique" if charge > 0.9 else "normale"` devient `let etat = if charge > 0.9 { "critique" } else { "normale" };`. Même idée, l'ordre naturel en plus.

### 3.3 `loop`, `while`, et `break` avec une valeur

Rust a trois boucles, une par usage. **`loop`** tourne pour toujours, jusqu'à un `break`. **`while`** tourne tant qu'une condition est vraie. **`for`** parcourt une suite de valeurs (3.4).

```rust
fn main() {
    let mut tentative = 0;
    let delai = loop {
        tentative += 1;
        let delai = 100 * 2_u32.pow(tentative);
        if delai > 1000 {
            break delai;
        }
        println!("tentative {tentative} : attente {delai} ms");
    };
    println!("abandon à {delai} ms");

    let mut restant = 3;
    while restant > 0 {
        println!("décollage dans {restant}");
        restant -= 1;
    }
}
```

```
$ cargo run
tentative 1 : attente 200 ms
tentative 2 : attente 400 ms
tentative 3 : attente 800 ms
abandon à 1600 ms
décollage dans 3
décollage dans 2
décollage dans 1
```

`loop` est la bonne boucle quand la condition d'arrêt est au milieu du corps : relancer une connexion, lire des lignes jusqu'à la fin, attendre une réponse valide. Et comme tout le reste, c'est une expression : `break delai;` sort de la boucle **en rendant une valeur**, qui devient celle de `loop`. Pas besoin de déclarer `delai` avant la boucle avec une valeur bidon pour le remplir dedans.

Pourquoi un mot-clé pour la boucle infinie, alors que `while true` existe ? Parce que le compilateur sait qu'une `loop` sans `break` ne se termine jamais, et qu'une variable affectée avant chaque `break` est forcément initialisée après la boucle. Avec `while true`, il ne fait pas ce raisonnement, et il t'avertit de toute façon : `denote infinite loops with loop { ... }`.

`continue` passe au tour suivant, dans les trois boucles. Il n'y a ni `do ... while`, ni `for (i = 0; i < n; i++)` à la C.

### 3.4 `for` sur des plages et des itérateurs

Le `for` de Rust ne connaît qu'une forme : `for x in quelque_chose`, où `quelque_chose` sait produire des valeurs l'une après l'autre (un *itérateur*, chapitre 5). Les cas que tu écriras tous les jours :

```rust
fn main() {
    for i in 0..3 {
        print!("{i} ");
    }
    println!();
    for i in 1..=3 {
        print!("{i} ");
    }
    println!();
    for i in (0..10).step_by(3) {
        print!("{i} ");
    }
    println!();
    for i in (1..=3).rev() {
        print!("{i} ");
    }
    println!();

    let hotes = vec!["web01", "web02", "db01"];
    for h in &hotes {
        print!("{h} ");
    }
    println!();
    for (i, h) in hotes.iter().enumerate() {
        println!("{i}: {h}");
    }
    for c in "été".chars() {
        print!("[{c}]");
    }
    println!();
    println!("{} hôtes", hotes.len());
}
```

```
$ cargo run
0 1 2 
1 2 3 
0 3 6 9 
3 2 1 
web01 web02 db01 
0: web01
1: web02
2: db01
[é][t][é]
3 hôtes
```

`0..3` est une **plage** demi-ouverte, 0 compris, 3 exclu, comme le `range(3)` de Python. `1..=3` est fermée, 3 compris. Une plage est une valeur comme une autre, avec des méthodes : `step_by` pour avancer de plusieurs pas, `rev` pour la parcourir à l'envers.

Sur une collection, `for h in &hotes` parcourt **en empruntant** : chaque `h` est une référence vers un élément, et `hotes` reste utilisable après la boucle, comme le montre la dernière ligne. Sans le `&`, `for h in hotes` **consomme** le vecteur : chaque élément sort de `hotes`, qui n'existe plus après la boucle. Cette différence est le sujet du chapitre 4 ; retiens pour l'instant le `&` devant la collection.

Pour avoir l'indice en plus de l'élément, `enumerate()` rend des paires `(indice, élément)`, que le `for` déstructure directement. C'est l'équivalent du `for i, h := range hotes` de Go et du `enumerate` de Python. Ce que tu n'écris pas, c'est `for i in 0..hotes.len()` suivi de `hotes[i]` : ça marche, mais chaque accès par indice est vérifié contre la longueur, et clippy te le reproche (`needless_range_loop`, vu au chapitre 1). L'itérateur, lui, ne peut pas sortir des bornes.

**Venant de Go :** Go n'a qu'une boucle, `for`, déclinée en quatre formes (avec trois clauses, avec une condition, infinie, et `range`). Rust en a trois, une par intention : `loop` pour `for { }`, `while` pour `for cond { }`, `for ... in` pour `range`. La forme à trois clauses n'existe pas : `for i := 0; i < n; i++` s'écrit `for i in 0..n`.

**Venant du C :** pas de `for (init; cond; incr)`, et c'est un choix. La moitié des bugs de boucle du C (le `<=` au lieu de `<`, l'indice modifié dans le corps, le dépassement d'une case) viennent de cette forme. Une plage `0..n` ne peut pas déborder, et un itérateur sur un tableau ne peut pas en sortir.

### 3.5 Étiquettes de boucles

Dans deux boucles imbriquées, `break` sort de la plus intérieure. Pour sortir des deux d'un coup, ou passer au tour suivant de la boucle extérieure, on **étiquette** la boucle avec un nom précédé d'une apostrophe :

```rust
fn main() {
    let grille = [[3, 8, 1], [9, 4, 7], [2, 6, 5]];
    let mut trouve = None;
    'lignes: for (l, ligne) in grille.iter().enumerate() {
        for (c, &valeur) in ligne.iter().enumerate() {
            if valeur == 4 {
                trouve = Some((l, c));
                break 'lignes;
            }
        }
    }
    println!("{trouve:?}");

    'serveurs: for s in ["web01", "db01", "web02"] {
        for port in [22, 80, 5432] {
            if s.starts_with("db") && port == 80 {
                continue 'serveurs;
            }
            print!("{s}:{port} ");
        }
    }
    println!();
}
```

```
$ cargo run
Some((1, 1))
web01:22 web01:80 web01:5432 db01:22 web02:22 web02:80 web02:5432
```

`break 'lignes` sort des deux boucles dès que la valeur est trouvée. `continue 'serveurs` abandonne le serveur en cours et passe au suivant : `db01` n'a été testé que sur le port 22. Le `&valeur` du motif dit « chaque élément est une référence, donne-moi directement l'entier pointé » ; tu reverras ce genre de motif au chapitre 4. Et `Some((l, c))` est une `Option`, dont on parle en 3.8.

C'est la même idée que les étiquettes de Go (`Lignes:` devant la boucle, puis `break Lignes`), avec une apostrophe devant le nom. Le C n'a que `goto` pour ça. Et en Python, on lève une exception ou on met une variable drapeau.

### 3.6 `match` : le `switch` qui ne laisse rien passer

`match` compare une valeur à une suite de **motifs**, de haut en bas, et exécute le bras du premier qui correspond :

```rust
fn categorie(code: u16) -> &'static str {
    match code {
        200 => "ok",
        301 | 302 | 307 | 308 => "redirection",
        400..=499 => "erreur client",
        500..=599 => "erreur serveur",
        _ => "autre",
    }
}

fn main() {
    for code in [200, 302, 404, 503, 101] {
        println!("{code} : {}", categorie(code));
    }
}
```

```
$ cargo run
200 : ok
302 : redirection
404 : erreur client
503 : erreur serveur
101 : autre
```

Chaque bras s'écrit `motif => expression`. `|` combine plusieurs motifs, `400..=499` est une plage, `_` attrape tout le reste. Il n'y a pas de `break` à écrire et pas de passage au bras suivant : un seul bras s'exécute. Et `match` est une expression : la fonction entière tient en un `match`, dont la valeur est le résultat. (`&'static str` est le type d'une chaîne littérale, qui vit aussi longtemps que le programme ; le chapitre 4 explique ce `'static`.)

Ce qui distingue `match` de tous les `switch` que tu connais : **le compilateur vérifie que les bras couvrent toutes les valeurs possibles**. Retire le `_` de l'exemple, et ça ne compile plus, parce qu'un `u16` peut valoir 101. C'est encore plus utile sur un type qui n'a qu'une poignée de valeurs, comme `Ordering`, que rend la méthode `cmp` (`Less`, `Equal` ou `Greater`) :

```rust
use std::cmp::Ordering;

fn main() {
    let secret = 42;
    let essai = 50;
    let message = match essai.cmp(&secret) {
        Ordering::Less => "plus grand",
        Ordering::Greater => "plus petit",
    };
    println!("{message}");
}
```

```
$ cargo build
error[E0004]: non-exhaustive patterns: `std::cmp::Ordering::Equal` not covered
 --> src/main.rs:6:25
  |
6 |     let message = match essai.cmp(&secret) {
  |                         ^^^^^^^^^^^^^^^^^^ pattern `std::cmp::Ordering::Equal` not covered
  |
  = note: the matched value is of type `std::cmp::Ordering`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
  |
8 ~         Ordering::Greater => "plus petit",
9 ~         std::cmp::Ordering::Equal => todo!(),
  |
```

Le cas d'égalité a été oublié, et le compilateur le dit, en nommant la valeur manquante. Le même raisonnement marche sur les plages d'entiers : avec `0..=9` et `10..=99` sur un `u8`, il répond `pattern 100_u8..=u8::MAX not covered`. Pourquoi c'est si précieux ? Parce que le jour où tu ajoutes une valeur à un `enum` (chapitre 6), chaque `match` qui ne la traite pas devient une erreur de compilation, et le compilateur te donne la liste des endroits à modifier. Dans un `switch` C ou Go, la nouvelle valeur passe en silence dans le `default`, ou nulle part.

D'où un conseil : sur tes propres `enum`, évite le `_` quand tu peux énumérer les cas. Le `_` dit « et tout le reste, y compris ce qu'on ajoutera plus tard », ce qui éteint la vérification.

**Venant de Go :** le `switch` de Go a déjà corrigé le pire du C (pas de passage implicite au cas suivant, sauf `fallthrough`). Le `match` de Rust va plus loin dans deux directions : il est exhaustif, vérifié, là où un `switch` Go sans `default` ne fait simplement rien pour une valeur oubliée ; et il compare des **motifs** (plages, tuples, structures, variantes d'`enum` avec leurs données), pas seulement des valeurs. Le `switch` sans expression de Go (`switch { case x > 0: ... }`) s'écrit avec des gardes, en 3.7, ou avec une chaîne de `if`.

**Venant du C :** pas de `break` oublié qui fait tomber dans le cas suivant, pas de `case` limité aux constantes entières. `match` marche sur les chaînes, les tuples, les plages, les `enum`. Et pas de `default` obligatoire pour faire taire `-Wswitch` : le compilateur sait si tu as tout couvert.

**Venant de Python :** `match` existe depuis Python 3.10, avec presque la même syntaxe de motifs. La différence est que Python ne vérifie rien : un cas oublié tombe dans le vide, à l'exécution.

### 3.7 Motifs : liaisons, gardes, tuples, chaînes

Un motif peut faire plus que comparer : il peut **lier** une partie de la valeur à un nom, et un bras peut ajouter une condition, la **garde**.

```rust
fn decrire(n: i32) -> String {
    match n {
        0 => "zéro".to_string(),
        chiffre @ 1..=9 => format!("le chiffre {chiffre}"),
        x if x < 0 => format!("{x} est négatif"),
        x if x % 2 == 0 => format!("{x}, pair"),
        x => format!("{x}, impair"),
    }
}

fn main() {
    for n in [0, 7, -3, 42, 99] {
        println!("{}", decrire(n));
    }

    let cible = ("db01", 5432);
    match cible {
        (_, 22) => println!("ssh"),
        ("db01", port) => println!("la base, sur le port {port}"),
        (hote, port) => println!("{hote}:{port}"),
    }

    let commande = "restart";
    match commande {
        "start" | "restart" => println!("démarrage"),
        "stop" => println!("arrêt"),
        autre => println!("commande inconnue : {autre}"),
    }
}
```

```
$ cargo run
zéro
le chiffre 7
-3 est négatif
42, pair
99, impair
la base, sur le port 5432
démarrage
```

Ligne par ligne :

- `chiffre @ 1..=9` : la valeur doit être dans la plage, **et** elle est liée au nom `chiffre`. Sans le `@`, tu saurais que c'est un chiffre sans pouvoir t'en servir. Le labo 03 l'utilise pour accepter un essai de 1 à 100 en une ligne : `Ok(n @ 1..=100) => Some(n)`.
- `x if x < 0` : un nom seul est un motif qui accepte tout et lie la valeur, et la garde `if` ajoute une condition. Si elle est fausse, on passe au bras suivant.
- `x` tout seul en dernier : accepte tout ce qui reste. C'est un `_` qui garde la valeur.
- `(_, 22)`, `("db01", port)` : les motifs de tuple déstructurent, chaque position a son propre motif.
- Sur une `&str`, chaque bras compare la chaîne entière.

L'ordre compte : le premier bras qui correspond gagne. Mets `x => ...` en premier et les bras suivants ne seront jamais atteints ; le compilateur le signale d'ailleurs (`unreachable pattern`). Une limite à connaître : les gardes ne comptent pas pour l'exhaustivité. Le compilateur ne sait pas que `x if x < 0` et `x if x >= 0` couvrent tout : il répond `E0004`, `pattern i32::MIN..=i32::MAX not covered`, et exige un dernier bras sans garde.

### 3.8 `Option`, premier contact : `.get()`

Que rend `ports[10]` sur un tableau de trois éléments ? Le chapitre 2 a répondu : une panique. Quand un indice peut être hors bornes et que ce n'est pas un bug, on demande poliment avec `.get()`, qui rend une **`Option`** :

```rust
fn main() {
    let ports = [22, 80, 443];
    println!("{:?}", ports.get(1));
    println!("{:?}", ports.get(10));

    match ports.get(10) {
        Some(p) => println!("port {p}"),
        None => println!("pas de onzième port"),
    }

    if let Some(p) = ports.get(2) {
        println!("le troisième est {p}");
    }

    let premier = ports.first().copied().unwrap_or(0);
    println!("premier : {premier}");
}
```

```
$ cargo run
Some(80)
None
pas de onzième port
le troisième est 443
premier : 22
```

`Option<T>` a deux valeurs possibles : `Some(valeur)`, qui contient une valeur de type `T`, ou `None`, qui n'en contient pas. C'est la façon qu'a Rust de dire « peut-être rien », et c'est la seule : **il n'y a pas de `null` en Rust**. Une fonction qui peut ne rien trouver le dit dans son type de retour, et l'appelant ne peut pas utiliser la valeur sans avoir traité le cas `None`. Tu l'as déjà croisée avec `find` et `checked_add` au chapitre 2 ; tu la croiseras partout.

Pour sortir la valeur, le `match` traite les deux cas. `unwrap_or(0)` donne la valeur ou un défaut. `copied()` transforme l'`Option<&i32>` que rend `first()` (une référence vers l'élément du tableau) en `Option<i32>` (une copie de l'entier). Et `if let`, la forme courte, est la section suivante. Le type lui-même, ses méthodes et la façon de l'utiliser dans tes propres fonctions sont au chapitre 6.

**Venant de Go :** `v, ok := m[k]` et les fonctions qui rendent un pointeur `nil` deviennent une `Option`. La différence de fond : en Go, rien n'empêche d'utiliser `v` sans regarder `ok`, ou de déréférencer le `nil`. En Rust, la valeur est *dans* le `Some`, et il faut un motif pour l'en sortir. Le `panic: runtime error: invalid memory address or nil pointer dereference` n'a pas d'équivalent.

### 3.9 `if let`, `let else` et les chaînes de `let`

Un `match` qui ne s'intéresse qu'à un seul cas est verbeux : deux bras, dont un `_ => {}` qui ne fait rien. **`if let`** est la forme courte. `if let Some(p) = ports.get(2) { ... }` se lit « si `ports.get(2)` correspond au motif `Some(p)`, exécute le bloc avec `p` ». Un `else` est permis, pour le cas où le motif ne correspond pas.

Son cousin **`while let`** boucle tant que le motif correspond :

```rust
fn main() {
    let mut pile = vec![1, 2, 3];
    while let Some(sommet) = pile.pop() {
        print!("{sommet} ");
    }
    println!();
}
```

```
$ cargo run
3 2 1 
```

`pop()` rend `Some(dernier)` tant que le vecteur n'est pas vide, puis `None`, ce qui arrête la boucle.

**`let else`** fait l'inverse de `if let` : il lie la valeur pour **toute la suite** de la fonction, et le bloc `else` traite le cas où le motif ne correspond pas. C'est la forme idéale pour les sorties anticipées, qui évite d'empiler les niveaux d'indentation :

```rust
fn port_de(ligne: &str) -> u16 {
    let Some((_, texte)) = ligne.split_once(':') else {
        return 0;
    };
    let Ok(port) = texte.parse::<u16>() else {
        return 0;
    };
    port
}

fn main() {
    for ligne in ["db01:5432", "web01", "cache01:huit"] {
        println!("{ligne} -> {}", port_de(ligne));
    }
}
```

```
$ cargo run
db01:5432 -> 5432
web01 -> 0
cache01:huit -> 0
```

`split_once(':')` rend `Some((avant, après))` ou `None` s'il n'y a pas de `:`. Le premier `let else` sort la paire, ou quitte la fonction. Le second fait la même chose avec le `Result` de `parse` : un motif `Ok(port)` marche exactement comme `Some(p)`. Après les deux, `port` est un `u16` ordinaire, au niveau d'indentation de la fonction. Le bloc `else` **doit sortir** (par `return`, `break`, `continue` ou une panique), parce qu'après lui, le code suppose que la variable existe :

```
$ cargo build
error[E0308]: `else` clause of `let...else` does not diverge
 --> src/main.rs:3:26
  |
3 |       let Some(n) = x else {
  |  __________________________^
4 | |         println!("rien");
5 | |     };
  | |_____^ expected `!`, found `()`
  |
  = help: try adding a diverging expression, such as `return` or `panic!(..)`
```

Le type `!` du message est celui d'une expression qui ne rend jamais la main (`return`, `break`, `panic!`).

Dernier outil, propre à l'édition 2024 : les **chaînes de `let`** (*let chains*), qui combinent plusieurs `if let` et des conditions ordinaires avec `&&`, dans un seul `if` :

```rust
fn main() {
    let lignes = ["port=5432", "port=80", "hote=db01", "port=abc"];
    for ligne in lignes {
        if let Some(valeur) = ligne.strip_prefix("port=")
            && let Ok(port) = valeur.parse::<u16>()
            && port >= 1024
        {
            println!("port non privilégié : {port}");
        } else {
            println!("ignoré : {ligne}");
        }
    }
}
```

```
$ cargo run
port non privilégié : 5432
ignoré : port=80
ignoré : hote=db01
ignoré : port=abc
```

Chaque maillon peut utiliser les noms liés par les précédents : `valeur` sert au `parse`, `port` à la comparaison. Le bloc ne s'exécute que si tout correspond. Avant, il fallait trois `if` imbriqués. Les chaînes de `let` sont stables depuis Rust 1.88, mais seulement en édition 2024 ; dans une crate en 2021, le compilateur refuse avec `let chains are only allowed in Rust 2024 or later`. Tu en verras donc encore peu dans le code existant.

Lequel choisir ? `match` quand plusieurs cas ont chacun leur traitement. `if let` quand un seul cas t'intéresse. `let else` quand le cas qui ne correspond pas est une sortie, et que la suite de la fonction a besoin de la valeur. Le labo 03 se sert des trois.

### 3.10 Lire l'entrée standard

Dernière pièce avant le labo : lire ce que l'utilisateur tape. `std::io::stdin().read_line` lit une ligne dans une `String` que tu lui prêtes :

```rust
use std::io;

fn main() {
    let mut total: u64 = 0;
    let mut ligne = String::new();
    loop {
        ligne.clear();
        let lus = io::stdin().read_line(&mut ligne).expect("lecture");
        if lus == 0 {
            break;
        }
        match ligne.trim().parse::<u64>() {
            Ok(n) => total += n,
            Err(e) => eprintln!("ignoré {:?} : {e}", ligne.trim()),
        }
    }
    println!("total : {total}");
}
```

```
$ printf '10\n20\ntrente\n\n12\n' | cargo run
ignoré "trente" : invalid digit found in string
ignoré "" : cannot parse integer from empty string
total : 42
```

Quatre détails, tous importants :

- `read_line(&mut ligne)` reçoit un **emprunt mutable** de la `String` : la fonction écrit dedans, mais la `String` reste à toi. Oublie le `mut` et le compilateur répond `types differ in mutability : expected &mut String, found &String`. Le chapitre 4 explique pourquoi il faut le dire.
- `read_line` **ajoute** la ligne à la fin de la `String`, il ne la remplace pas. Sans le `ligne.clear()`, la deuxième lecture donne `"10\n20\n"`, que `parse` refuse. C'est voulu (ça permet de réutiliser le tampon sans réallouer), et c'est un piège classique.
- La ligne lue contient son `\n` final (et le `\r\n` sous Windows). `trim()` l'enlève, avant tout `parse`.
- `read_line` rend un `Result<usize>` : le nombre d'octets lus, ou une erreur d'entrée-sortie. **Zéro octet signifie la fin de l'entrée** : Ctrl-D au clavier, ou la fin du fichier ou du tuyau. C'est ce qui permet au programme de s'arrêter proprement avec `printf ... |`. Le `expect` arrête le programme sur une vraie erreur, ce qui est acceptable ici en attendant le chapitre 8.

`eprintln!` écrit sur la sortie d'erreur, comme `fmt.Fprintln(os.Stderr, ...)` en Go : les messages de diagnostic n'y polluent pas la sortie standard, qu'on peut rediriger dans un fichier.

Quand tu veux juste parcourir toutes les lignes, `for ligne in io::stdin().lines()` est plus court : chaque `ligne` est un `Result<String>`, sans le `\n`, et la boucle s'arrête seule en fin d'entrée. `read_line` reste la bonne forme pour un dialogue, où tu lis une réponse, réagis, puis lis la suivante.

**Venant de Go :** `bufio.NewScanner(os.Stdin)` et sa boucle `for sc.Scan()` deviennent `io::stdin().lines()`. `reader.ReadString('\n')` devient `read_line`, avec le même `\n` final à retirer.

**Venant de Python :** `input()` rend la ligne sans le `\n` et lève `EOFError` en fin d'entrée. `read_line` garde le `\n` et rend 0. `for ligne in sys.stdin:` devient `for ligne in io::stdin().lines()`.

### 3.11 Pour le labo

Le [labo 03](../labs/03-devine-nombre/README.md) est le jeu du nombre à deviner. La bibliothèque juge un essai avec un `match` sur `cmp` (retire un bras pour lire `E0004` de tes yeux), lit une ligne avec un seul `match` et le motif `Ok(n @ 1..=100)`, rejoue une partie avec un `for`, un `let else` et un `break`, et choisit le message avec une garde. Le programme principal tire le secret avec la crate `rand`, ta première dépendance, et boucle sur l'entrée standard avec `loop` et `read_line` jusqu'à la victoire ou la fin de l'entrée. Les tests ne dépendent ni du hasard ni du clavier : le secret et les essais sont passés en paramètres.

### À retenir

- Presque tout est une expression : un bloc vaut sa dernière expression sans point-virgule, et une fonction rend la sienne sans `return`.
- `if` est une expression : pas d'opérateur ternaire, les deux branches ont le même type, et la condition est un vrai `bool`.
- `loop` pour la boucle dont la sortie est au milieu (et `break valeur` pour en sortir avec un résultat), `while` pour une condition, `for x in ...` pour tout le reste.
- `for i in 0..n`, `1..=n`, `.rev()`, `.step_by(k)` ; `for x in &v` emprunte la collection, `for (i, x) in v.iter().enumerate()` donne l'indice en plus.
- Les étiquettes `'nom:` permettent `break 'nom` et `continue 'nom` dans des boucles imbriquées.
- `match` est exhaustif : un cas oublié est l'erreur `E0004`, qui nomme la valeur manquante. Évite le `_` sur tes propres `enum`.
- Les motifs combinent valeurs, `|`, plages `1..=9`, liaisons `n @ 1..=9`, tuples, `_`, et gardes `x if x < 0`. Le premier bras qui correspond gagne.
- `Option<T>` vaut `Some(valeur)` ou `None`, et il n'y a pas de `null` : `.get(i)` au lieu de `[i]` quand l'indice peut manquer.
- `if let` pour un seul cas, `let else` pour une sortie anticipée qui garde la valeur, et les chaînes `if let ... && let ... && cond` en édition 2024.
- `read_line(&mut ligne)` ajoute à la `String` : `clear()` avant, `trim()` après, et 0 octet lu signifie la fin de l'entrée.

---

← [2. Variables, types, chaînes](02-variables-types-chaines.md) · [Sommaire](../README.md) · [4. Possession et emprunts : le cœur de Rust](04-possession-et-emprunts.md) →
