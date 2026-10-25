# 2. Variables, types, chaînes

*Rust de zéro à la prod : chapitre 2 sur 16.* ← [1. Installer, outiller, lancer](01-installer-et-outiller.md) · [Sommaire](../README.md) · [3. Contrôle : if, loop, while, for, match](03-controle.md) →

Rust est typé statiquement, comme le C et le Go : chaque variable a un type fixé à la compilation. Comme en Go, tu l'écris rarement, parce que le compilateur le déduit. Mais Rust ajoute deux choses que tu n'as vues dans aucun des trois langages : une variable est **immuable par défaut**, et il n'y a **aucune conversion implicite**, même d'un `i32` vers un `i64`. Ce chapitre pose ces règles, fait le tour des types de base, montre ce qui se passe quand un entier déborde (la réponse dépend du profil de compilation), puis s'attarde sur les chaînes. Rust en a deux, `String` et `&str`, et comprendre pourquoi est le premier pas vers le chapitre 4.

### 2.1 `let`, et l'immutabilité par défaut

`let` déclare une variable. Le type est déduit de la valeur :

```rust
fn main() {
    let port = 5432;
    println!("port = {port}");
    port = 5433;
    println!("port = {port}");
}
```

```
$ cargo build
error[E0384]: cannot assign twice to immutable variable `port`
 --> src/main.rs:4:5
  |
2 |     let port = 5432;
  |         ---- first assignment to `port`
3 |     println!("port = {port}");
4 |     port = 5433;
  |     ^^^^^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut port = 5432;
  |         +++
```

Une variable déclarée avec `let` ne change plus. Pour qu'elle change, tu le dis avec `mut` :

```rust
fn main() {
    let mut connexions = 0;
    connexions += 1;
    connexions += 1;
    println!("connexions = {connexions}");
}
```

```
$ cargo run
connexions = 2
```

Pourquoi ce choix, alors que tous les autres langages font l'inverse ? Parce que la plupart des variables ne changent jamais après leur création, et que celles qui changent sont celles où se cachent les bugs. Avec `mut` obligatoire, le lecteur sait d'un coup d'œil quelles valeurs bougent : un `let` sans `mut` est une promesse vérifiée par le compilateur. Et cette distinction entre « je lis » et « je modifie » est exactement celle sur laquelle repose tout le système d'emprunts du chapitre 4. Le compilateur te prévient d'ailleurs dans l'autre sens : un `mut` inutile est un avertissement (`variable does not need to be mutable`).

Lis bien le message d'erreur : le numéro (`E0384`), la position, les deux lignes concernées, et une suggestion de correction prête à appliquer. Les erreurs de Rust sont longues parce qu'elles expliquent. `rustc --explain E0384` en donne une page complète, avec un exemple.

**Venant de Go :** `let` est le `:=` de Go, et `let mut` est ce que `:=` fait toujours. Il n'y a pas de valeur zéro en Rust : `let x: i32;` sans valeur est permis, mais lire `x` avant de lui avoir donné une valeur ne compile pas (`E0381: used binding isn't initialized`). Pas de variable non initialisée, comme en Go, mais par une vérification du compilateur plutôt que par un zéro implicite. Et une variable inutilisée n'est qu'un avertissement, pas une erreur : préfixe-la par `_` pour le faire taire (`let _brouillon = 3;`).

**Venant du C :** c'est le `const` du C appliqué par défaut. `let x = 5;` est `const int x = 5;`, et `let mut x = 5;` est le `int x = 5;` que tu écrivais partout.

### 2.2 Le shadowing : redéclarer sous le même nom

Rust permet de redéclarer une variable avec le même nom. La nouvelle **masque** l'ancienne, et peut avoir un autre type :

```rust
fn main() {
    let port = "5432";
    let port: u16 = port.parse().expect("un port");
    let port = port + 1;
    println!("{port}");

    let x = 10;
    {
        let x = x * 2;
        println!("dans le bloc : {x}");
    }
    println!("après le bloc : {x}");
}
```

```
$ cargo run
5433
dans le bloc : 20
après le bloc : 10
```

Ce n'est pas une modification : ce sont trois variables différentes, qui portent le même nom. La première est une `&str`, la deuxième un `u16`, la troisième un autre `u16`. Dans le bloc, `x` masque l'`x` extérieur jusqu'à l'accolade fermante, puis l'ancien réapparaît, intact.

À quoi ça sert ? Au cas qui revient tout le temps : une valeur passe par plusieurs formes (le texte lu, le nombre analysé, le nombre validé), et tu n'as pas envie d'inventer `port_texte`, `port_brut`, `port_final`. Le shadowing garde un seul nom et laisse chaque étape immuable. C'est idiomatique, tu le verras partout. Ne t'en sers pas pour réutiliser un nom pour une chose sans rapport : là, il embrouille.

**Venant de Go :** le shadowing existe aussi en Go, mais seulement dans un bloc intérieur, et `go vet -shadow` le signale comme suspect. En Rust, dans le même bloc, c'est voulu.

### 2.3 `const` et `static`

`const` déclare une constante calculée à la compilation. Le type est **obligatoire**, et le nom s'écrit en majuscules :

```rust
const PORT_POSTGRES: u16 = 5432;
const KO: u64 = 1024;
const MO: u64 = KO * 1024;
static VERSION: &str = "1.4.2";

fn main() {
    println!("{PORT_POSTGRES} {MO} {VERSION}");
}
```

```
$ cargo run
5432 1048576 1.4.2
```

Une `const` n'a pas d'adresse : le compilateur recopie sa valeur à chaque endroit où elle sert, comme un `#define` typé. Une `static` est une variable globale unique, à une adresse fixe, qui vit pendant tout le programme. Dans 95 % des cas, tu veux `const`. `static` sert quand l'adresse compte (une grande table qu'on ne veut pas dupliquer) ou pour un état global, et une `static mut` modifiable ne se lit qu'avec `unsafe` (chapitre 10), parce que deux threads pourraient y écrire en même temps. Rust n'aime pas les variables globales modifiables, et il a raison.

**Venant de Go :** pas d'`iota`, et les constantes ne sont pas « non typées » : `const KO: u64` est un `u64`, point. Les énumérations se font avec `enum`, un vrai type fermé, au chapitre 6.

### 2.4 Les types scalaires

| Type | Taille | Usage |
|---|---|---|
| `i8`, `i16`, `i32`, `i64`, `i128` | ce que le nom dit | entiers signés ; `i32` par défaut |
| `u8`, `u16`, `u32`, `u64`, `u128` | idem | non signés : octets, ports, tailles, identifiants |
| `isize`, `usize` | taille d'un pointeur (64 bits sur ta machine) | `usize` : indices, longueurs, tout ce qui compte des éléments en mémoire |
| `f32`, `f64` | IEEE 754 | `f64` par défaut |
| `bool` | 1 octet | `true`, `false` |
| `char` | 4 octets | un point de code Unicode : `'a'`, `'é'`, `'€'` |

Les choix par défaut du compilateur, quand rien ne l'oriente : `i32` pour un entier, `f64` pour un flottant.

```rust
use std::mem::size_of;

fn main() {
    let a = 42;
    let b = 0.75;
    let c = 'é';
    let d = true;
    let e = 255u8;
    let f = 1_000_000_i64;
    println!("{a} {b} {c} {d} {e} {f}");
    println!(
        "i32={} f64={} char={} bool={} u8={} usize={}",
        size_of::<i32>(),
        size_of::<f64>(),
        size_of::<char>(),
        size_of::<bool>(),
        size_of::<u8>(),
        size_of::<usize>()
    );
    println!("{} {}", i32::MAX, u64::MAX);
    println!("{} {}", -7 / 2, -7 % 2);
    println!("{}", 0.1 + 0.2);
}
```

```
$ cargo run
42 0.75 é true 255 1000000
i32=4 f64=8 char=4 bool=1 u8=1 usize=8
2147483647 18446744073709551615
-3 -1
0.30000000000000004
```

Un suffixe fixe le type d'un littéral (`255u8`, `1_000_000_i64`, `2.5f32`), et le `_` sépare les chiffres où tu veux, il est ignoré. La division entière tronque vers zéro, comme en C et en Go. Le `f64` a les arrondis que tu connais : pour de l'argent, compte en centimes dans un entier.

Le type que tu vas rencontrer le plus et qui surprend : **`usize`**. Tout ce qui est une longueur ou un indice en mémoire est un `usize` : `v.len()`, `s.len()`, `tableau[i]`. Un indice ne peut pas être négatif, et sa taille suit celle des pointeurs. Si tu as un `i32` et que tu veux indexer, il faut convertir (2.7), et c'est voulu.

Le `char` est un **point de code Unicode**, quatre octets, toujours. Ce n'est pas un octet comme le `char` du C, et ce n'est pas un caractère tel que l'œil le voit (un drapeau ou un emoji de famille en contiennent plusieurs). Un littéral `'a'` entre apostrophes est un `char`, `"a"` entre guillemets est une chaîne.

**Venant du C :** pas de `int` dont la taille dépend de la plateforme, pas de `long` ni `short`, pas de `unsigned` en préfixe. `i32` est `int32_t`, `u8` est `uint8_t`, `usize` est `size_t`. Et pas de promotion implicite : un `u8` plus un `i32` ne compile pas, il faut convertir l'un des deux.

**Venant de Go :** `int` devient `i64` (ou `isize`), `byte` devient `u8`, `rune` devient `char`, `float64` devient `f64`. L'entier par défaut est `i32` et pas 64 bits : ça ne pose presque jamais de problème, parce que l'inférence choisit le type d'après l'usage (si la variable finit dans une fonction qui attend un `u64`, elle est `u64` dès le départ).

### 2.5 Le débordement : ça dépend du profil

Que vaut un `u8` à 250 auquel on ajoute 10 ? En C, c'est 4 (pour un non signé) ou un comportement indéfini (pour un signé). En Go, c'est 4. En Python, c'est 260. En Rust, c'est d'abord une erreur :

```rust
fn main() {
    let mut compteur: u8 = 250;
    for _ in 0..10 {
        compteur += 1;
    }
    println!("{compteur}");
}
```

```
$ cargo run

thread 'main' (55206922) panicked at src/main.rs:4:9:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ cargo run --release
4
```

En profil `dev`, chaque opération arithmétique est vérifiée, et un débordement **panique** : le programme s'arrête avec un message et la ligne fautive. En profil `release`, la vérification est retirée pour la vitesse, et l'entier **s'enroule** en silence, comme en C non signé. Ce n'est jamais un comportement indéfini, c'est défini et documenté : le complément à deux. L'idée est que tes tests, lancés en `dev`, attrapent les débordements, et que la production ne paie pas le prix des vérifications. Si tu veux les garder en release, `overflow-checks = true` dans `[profile.release]` le fait.

Quand le débordement est une possibilité réelle et pas un bug, ne compte pas sur le profil : dis ce que tu veux avec une méthode explicite.

```rust
fn main() {
    let x: u8 = 250;
    println!("{}", x.wrapping_add(10));
    println!("{:?}", x.checked_add(10));
    println!("{:?}", x.checked_add(5));
    println!("{}", x.saturating_add(10));
    println!("{:?}", x.overflowing_add(10));
    let y: u32 = 3;
    println!("{}", y.saturating_sub(5));
}
```

```
$ cargo run
4
None
Some(255)
255
(4, true)
0
```

| Méthode | Résultat en cas de débordement | Quand |
|---|---|---|
| `wrapping_add` | s'enroule (modulo 2^n) | sommes de contrôle, hachage, compteurs circulaires |
| `checked_add` | `None` (sinon `Some(valeur)`) | entrée utilisateur, tailles : tu veux savoir et réagir |
| `saturating_add` | reste bloqué au maximum (ou à 0) | compteurs, jauges, « temps restant » qui ne doit pas devenir énorme |
| `overflowing_add` | la valeur enroulée et un booléen | rare : arithmétique multi-précision |

Elles existent pour toutes les opérations (`_sub`, `_mul`, `_div`, `_pow`). `checked_` rend une `Option`, ta première rencontre avec ce type : `Some(valeur)` ou `None`, à traiter avant de s'en servir (chapitre 3 pour la manière, chapitre 6 pour le type). La soustraction de non signés est le cas classique : `3u32 - 5` panique en debug, et `saturating_sub` donne 0, ce qui est souvent ce que tu voulais dire. Le [labo 02](../labs/02-convertisseur/README.md) se sert de `checked_mul` pour qu'une durée absurde (`99999999999999h`) donne une erreur propre plutôt qu'un résultat faux.

### 2.6 Tuples et tableaux

Un **tuple** groupe quelques valeurs de types différents, sans leur donner de nom. Un **tableau** contient un nombre fixe de valeurs du même type, et ce nombre fait partie du type.

```rust
fn main() {
    let cible = ("db01", 5432, true);
    println!("{} {} {}", cible.0, cible.1, cible.2);
    let (hote, port, actif) = cible;
    println!("{hote}:{port} actif={actif}");
    println!("{cible:?}");

    let ports = [22, 80, 443];
    let zeros = [0u8; 4];
    println!("{} {:?} {:?}", ports.len(), ports, zeros);
    println!("{}", ports[1]);
    let i = ports.len();
    println!("{}", ports[i]);
}
```

```
$ cargo run
db01 5432 true
db01:5432 actif=true
("db01", 5432, true)
3 [22, 80, 443] [0, 0, 0, 0]
80

thread 'main' (55211492) panicked at src/main.rs:13:20:
index out of bounds: the len is 3 but the index is 3
```

Le tuple `(&str, i32, bool)` s'accède par position (`.0`, `.1`) ou se **déstructure** d'un coup avec `let (a, b, c) = ...`. Il sert surtout à rendre plusieurs valeurs d'une fonction, comme le `return a, b` de Go. Au-delà de trois champs, fais une `struct` (chapitre 6) : `cible.2` ne dit rien au lecteur, `cible.actif` si.

Le tableau `[i32; 3]` a une taille fixe connue à la compilation, et il vit sur la pile. `[0u8; 4]` répète une valeur. Chaque accès par indice est **vérifié** : hors bornes, le programme panique avec l'indice et la longueur, au lieu de lire la mémoire du voisin comme en C. Tu écriras peu de tableaux : la collection de tous les jours est le `Vec<T>`, un tableau qui grandit, au chapitre 5. `{:?}` affiche les tuples, les tableaux et les `Vec` (2.9).

### 2.7 Conversions : toujours explicites

Rust ne convertit jamais un type numérique en un autre tout seul. Jamais :

```rust
fn main() {
    let octets: u64 = 1536;
    let ko = octets / 1024.0;
    let a: i32 = 10;
    let b: i64 = 20;
    let c = a + b;
    println!("{ko} {c}");
}
```

```
$ cargo build
error[E0277]: cannot divide `u64` by `{float}`
 --> src/main.rs:3:21
  |
3 |     let ko = octets / 1024.0;
  |                     ^ no implementation for `u64 / {float}`

error[E0308]: mismatched types
 --> src/main.rs:6:17
  |
6 |     let c = a + b;
  |                 ^ expected `i32`, found `i64`
```

Même `i32` vers `i64`, qui ne peut rien perdre, n'est pas implicite. C'est volontaire : la règle est simple, sans exception à retenir, et chaque conversion se voit dans le code. Trois façons de convertir, de la plus brutale à la plus sûre :

```rust
fn main() {
    let octets: u64 = 1536;
    let ko = octets as f64 / 1024.0;
    println!("{ko}");

    let x = 3.99_f64;
    println!("{}", x as i32);
    println!("{}", -1.5_f64 as u8);
    println!("{}", 300.0_f64 as u8);
    println!("{}", 300_i32 as u8);
    println!("{}", -1_i32 as u32);

    let petit: u16 = 5432;
    let grand: u64 = u64::from(petit);
    let aussi: u64 = petit.into();
    println!("{grand} {aussi}");

    let n: i64 = 70_000;
    let r: Result<u16, _> = u16::try_from(n);
    println!("{r:?}");
    let r2 = u16::try_from(5432_i64);
    println!("{r2:?}");
}
```

```
$ cargo run
1.5
3
0
255
44
4294967295
5432 5432
Err(TryFromIntError(()))
Ok(5432)
```

**`as`** convertit entre types numériques, et ne se plaint jamais. D'un flottant vers un entier, il tronque vers zéro et **sature** aux bornes (`-1.5 as u8` donne 0, `300.0 as u8` donne 255). D'un entier vers un entier plus petit, il **garde les bits du bas** (`300 as u8` donne 44, c'est-à-dire 300 modulo 256). D'un signé vers un non signé, il réinterprète (`-1 as u32` donne 4294967295). C'est le cast du C, en mieux défini. Utilise-le pour entier vers flottant (`octets as f64`) et quand tu as vérifié la plage toi-même ; ailleurs, préfère les deux suivants.

**`From` et `Into`** convertissent quand c'est **sans perte** : `u16` vers `u64`, `i32` vers `i64`, `u8` vers `char`, `&str` vers `String`. `u64::from(petit)` et `petit.into()` font la même chose ; `into()` a besoin que le type cible soit connu (ici par l'annotation `: u64`). Si la conversion n'est pas sans perte (`u64` vers `u16`), `From` n'existe pas et ça ne compile pas : tu ne peux pas perdre de données par accident.

**`TryFrom`** convertit quand ça **peut échouer**, et rend un `Result` : `Ok(valeur)` si elle tient, `Err` sinon. C'est la bonne façon de transformer un `i64` lu dans un fichier en `u16` de port. Les traits `From`, `Into`, `TryFrom` reviennent au chapitre 7, où tu les implémenteras pour tes propres types.

**Venant de Go :** `float64(n)` devient `n as f64`. Go tronque aussi en silence (`int16(70000)` donne 4464) ; la différence est que Rust offre `try_from` pour ne pas le faire, et `from` pour signaler que c'est sans risque.

### 2.8 `String` et `&str` : deux types pour le texte

Voilà le point important du chapitre. Rust a deux types de chaînes, et tu vas les croiser à chaque ligne.

- **`String`** : une chaîne **possédée**, sur le tas, qui peut grandir. Elle a un propriétaire, et quand il disparaît, la mémoire est libérée.
- **`&str`** (dite « tranche de chaîne ») : une **vue empruntée** sur du texte qui appartient à quelqu'un d'autre. Un pointeur et une longueur, seize octets, rien à libérer.

Un littéral `"db01"` est une `&str` : le texte est gravé dans le binaire, et la variable n'en est qu'une vue. `String::from("db01")` ou `"db01".to_string()` copie ce texte sur le tas dans une `String` que tu possèdes et que tu peux modifier :

```rust
fn main() {
    let litteral: &str = "été";
    let mut possedee: String = String::from("db");
    possedee.push_str("01");
    possedee.push('!');
    println!("{litteral} {possedee}");

    println!("{}", litteral.len());
    println!("{}", litteral.chars().count());
    println!("{:?}", litteral.as_bytes());
    for (i, c) in litteral.char_indices() {
        print!("{i}:{c} ");
    }
    println!();
    println!("{}", &litteral[2..]);
    println!("{:?}", litteral.chars().nth(0));
}
```

```
$ cargo run
été db01!
5
3
[195, 169, 116, 195, 169]
0:é 2:t 3:é 
té
Some('é')
```

Pourquoi deux types ? Parce que la question « qui possède ce texte, et qui doit le libérer ? » est la question centrale de Rust, et que les deux types y répondent différemment. Une fonction qui ne fait que lire du texte prend une `&str` : elle accepte un littéral, une `String` (empruntée avec `&`), ou un morceau de l'une ou de l'autre, sans rien copier. Une structure qui doit garder du texte pour elle stocke une `String`. Le chapitre 4 explique le mécanisme ; pour l'instant, la règle pratique : **paramètres en `&str`, champs et valeurs de retour en `String`**.

Les deux sont **toujours de l'UTF-8 valide**, garanti par le type. Conséquences, les mêmes qu'en Go :

- `len()` compte les **octets** : `"été"` en fait 5, parce que `é` s'écrit `c3 a9`.
- `chars()` parcourt les caractères (des `char`), `char_indices()` donne en plus l'indice d'octet où chacun commence, `bytes()` ou `as_bytes()` donnent les octets.
- On ne peut **pas indexer** une chaîne par un entier : `s[0]` ne compile pas (`E0277: the type str cannot be indexed by {integer}`), parce que « le premier octet » et « le premier caractère » ne sont pas la même chose et que Rust refuse de choisir pour toi. `s.chars().nth(0)` donne le premier caractère, en parcourant.
- On peut **découper** par plage d'octets, `&s[2..]`, et le résultat est une `&str` qui regarde le même texte. Mais si la coupure tombe au milieu d'un caractère, le programme panique :

```rust
fn main() {
    let s = "été";
    println!("{}", &s[0..1]);
}
```

```
$ cargo run

thread 'main' (55215917) panicked at src/main.rs:3:22:
byte index 1 is not a char boundary; it is inside 'é' (bytes 0..2) of `été`
```

Go, lui, t'aurait rendu un octet invalide sans rien dire. Rust ne fabrique jamais une `&str` qui ne soit pas de l'UTF-8. Les indices sûrs viennent de `find`, de `char_indices` ou de la longueur d'un préfixe que tu as toi-même trouvé.

Les méthodes de tous les jours, sur `&str` (et donc aussi sur `String`, qui sait se prêter en `&str`) :

```rust
fn main() {
    let ligne = "  web01,db02, cache03 ,web01  ";
    println!("{:?}", ligne.trim());
    let morceaux: Vec<&str> = ligne.trim().split(',').collect();
    println!("{morceaux:?}");
    println!("{:?}", "  ERROR   disk   full  ".split_whitespace().collect::<Vec<_>>());
    println!("{}", ["a", "b"].join(" | "));
    println!("{}", ligne.contains("db"));
    println!("{}", "web01".starts_with("web"));
    println!("{:?}", "app.log".find('.'));
    println!("{:?}", "app.log".find('#'));
    println!("{}", "erreur".to_uppercase());
    println!("{}", "a-b-c".replace('-', "_"));
    println!("{}", "-".repeat(20));
    let hote = String::from("db01");
    let port = 5432;
    let adresse = hote + ":" + &port.to_string();
    println!("{adresse}");
    let brut = r"C:\logs\app.log";
    println!("{brut}");
}
```

```
$ cargo run
"web01,db02, cache03 ,web01"
["web01", "db02", " cache03 ", "web01"]
["ERROR", "disk", "full"]
a | b
true
true
Some(3)
None
ERREUR
a_b_c
--------------------
db01:5432
C:\logs\app.log
```

`trim`, `split` et `split_whitespace` ne copient rien : ils rendent des `&str` qui regardent la chaîne d'origine. `collect()` rassemble les morceaux dans un `Vec` (chapitre 5 pour les itérateurs). `find` rend une `Option<usize>` : `Some(indice)` ou `None`, pas de `-1` magique. `to_uppercase`, `replace` et `repeat` fabriquent une nouvelle `String`. `hote + ":"` **consomme** `hote` (on ne peut plus s'en servir après) et y ajoute une `&str` : c'est ta première rencontre avec un déplacement, et la raison pour laquelle on écrit plutôt `format!("{hote}:{port}")`. Les chaînes brutes `r"..."` n'interprètent pas les `\`, comme en Python.

**Venant de Python :** une `str` Python est une suite de caractères, et `len("été")` vaut 3. En Rust, c'est 5. Les méthodes ressemblent (`trim` pour `strip`, `split`, `replace`, `starts_with` pour `startswith`, `to_uppercase` pour `upper`), mais `s[0]` n'existe pas.

**Venant de Go :** `String` et `&str` sont deux visages de la `string` de Go. La `&str` est la `string` Go presque trait pour trait (un pointeur et une longueur, immuable, partageable sans copie). La `String` est un `strings.Builder` qui serait aussi une chaîne. Ce que Go règle avec le ramasse-miettes (qui garde le texte en vie tant que quelqu'un le regarde), Rust le règle avec deux types et des règles d'emprunt. Et les fonctions de `strings` deviennent des méthodes : `strings.TrimSpace(s)` s'écrit `s.trim()`.

**Venant du C :** une `&str` est un `const char *` qui connaîtrait sa longueur et garantirait l'UTF-8, sans `\0` final. Une `String` est le `malloc` + `realloc` + `free` que tu écrivais autour, avec le `free` placé par le compilateur.

### 2.9 `format!`, `println!` et le formatage

`println!` affiche et passe à la ligne, `print!` affiche sans passer à la ligne, `eprintln!` écrit sur la sortie d'erreur, et `format!` rend une `String`. Toutes prennent la même chaîne de format :

```rust
fn main() {
    let cibles = vec![("db01", 5432), ("web01", 80)];
    let charge = 0.7345;
    let nom = "web01";
    let largeur = 8;
    println!("{cibles:?}");
    println!("{cibles:#?}");
    println!("[{:5}] [{:<5}] [{:>5}] [{:^5}] [{:05}]", 42, 42, 42, 42, 42);
    println!("[{charge}] [{charge:.2}] [{charge:8.1}] [{charge:e}]");
    println!("[{nom}] [{nom:10}] [{nom:>10}] [{nom:^10}] [{nom:?}]");
    println!("[{nom:.<10}] [{:*^9}] [{nom:>largeur$}]", "ok");
    println!("{:x} {:X} {:o} {:b} {:#x} {:#b}", 255, 255, 8, 5, 255, 5);
    println!("{{}} s'écrit avec des accolades doublées");
}
```

```
$ cargo run
[("db01", 5432), ("web01", 80)]
[
    (
        "db01",
        5432,
    ),
    (
        "web01",
        80,
    ),
]
[   42] [42   ] [   42] [ 42  ] [00042]
[0.7345] [0.73] [     0.7] [7.345e-1]
[web01] [web01     ] [     web01] [  web01   ] ["web01"]
[web01.....] [***ok****] [   web01]
ff FF 10 101 0xff 0b101
{} s'écrit avec des accolades doublées
```

| Format | Effet |
|---|---|
| `{}` | l'affichage « pour humain » (le trait `Display`) |
| `{:?}` | l'affichage de débogage (le trait `Debug`) : chaînes entre guillemets, collections, tuples |
| `{:#?}` | idem, sur plusieurs lignes, indenté : pour les grosses structures |
| `{x}`, `{x:?}`, `{x:>8}` | la variable `x` en ligne, avec ou sans options |
| `{:10}`, `{:<10}`, `{:>10}`, `{:^10}` | largeur 10, alignée à gauche, à droite, centrée |
| `{:05}`, `{:.<10}` | remplir avec des zéros, avec un autre caractère |
| `{:.2}`, `{:8.1}` | deux décimales ; largeur 8 avec une décimale |
| `{:x}`, `{:X}`, `{:o}`, `{:b}`, `{:e}` | hexadécimal, octal, binaire, notation scientifique ; `#` ajoute `0x` ou `0b` |
| `{:>largeur$}` | largeur lue dans une variable |
| `{{` et `}}` | une accolade littérale |

Sans alignement précisé, un nombre s'aligne à droite et un texte à gauche, comme dans un tableur. La largeur compte des caractères, pas des octets : `"été"` dans `{:<5}` occupe bien cinq colonnes. `{:?}` est ton meilleur ami pour déboguer : il marche sur presque tous les types de la std, et sur les tiens dès que tu ajoutes `#[derive(Debug)]` (chapitre 6). `{}` ne marche que sur les types qui ont choisi un affichage pour humain : un `Vec` n'en a pas, et `println!("{}", v)` ne compile pas (`E0277: Vec<i32> doesn't implement std::fmt::Display`), ce qui est une invitation à mettre `{:?}`.

**Venant de Go :** `%v` devient `{}` ou `{:?}`, `%+v` devient `{:#?}`, `%q` devient `{:?}` sur une chaîne, `%-10s` devient `{:<10}`, `%5d` devient `{:5}`, `%.2f` devient `{:.2}`. Il n'y a pas de `%T` : pour connaître le type d'une variable, ton éditeur te l'affiche, ou `std::any::type_name`. Et `fmt.Sprintf` devient `format!`. La différence de fond : une erreur de format en Go donne `%!d(string=oups)` à l'exécution, en Rust elle ne compile pas.

### 2.10 Du texte vers un nombre : `parse`, et un premier `Result`

La conversion inverse de `to_string()` est `parse()`. Un texte peut ne pas être un nombre, donc `parse` ne rend pas un nombre mais un **`Result`** : `Ok(nombre)` ou `Err(raison)`.

```rust
fn main() {
    let bon = "42".parse::<i32>();
    let mauvais = "4x2".parse::<i32>();
    let trop_grand = "300".parse::<u8>();
    let vide = "".parse::<i32>();
    println!("{bon:?}");
    println!("{mauvais:?}");
    println!("{trop_grand:?}");
    println!("{vide:?}");

    let port: u16 = "5432".parse().expect("un port valide");
    println!("{}", port + 1);

    match "8o80".parse::<u16>() {
        Ok(p) => println!("port {p}"),
        Err(e) => println!("pas un port : {e}"),
    }
    let f: f64 = "0.75".parse().unwrap();
    let b: bool = "true".parse().unwrap();
    println!("{f} {b} {}", 8080.to_string() + "/tcp");
}
```

```
$ cargo run
Ok(42)
Err(ParseIntError { kind: InvalidDigit })
Err(ParseIntError { kind: PosOverflow })
Err(ParseIntError { kind: Empty })
5433
pas un port : invalid digit found in string
0.75 true 8080/tcp
```

`parse` doit savoir vers quel type analyser. Soit tu le dis avec le *turbofish*, `parse::<i32>()` (le nom vient de la forme `::<>`), soit tu annotes la variable, `let port: u16 = ...parse()`, et l'inférence remonte jusqu'à `parse`. Si rien ne le dit :

```
$ cargo build
error[E0284]: type annotations needed
 --> src/main.rs:2:9
  |
2 |     let n = "42".parse().unwrap();
  |         ^        ----- type must be known at this point
  |
help: consider giving `n` an explicit type
  |
2 |     let n: /* Type */ = "42".parse().unwrap();
  |          ++++++++++++
```

Pour récupérer le nombre, trois manières, dans l'ordre où tu les rencontreras. `unwrap()` prend la valeur ou fait paniquer le programme : acceptable dans un test ou un essai, jamais sur une entrée utilisateur. `expect("message")` fait pareil avec ton message. Et `match`, qui traite les deux cas proprement : c'est le sujet du chapitre 3. Le chapitre 8 est entièrement consacré à `Result`, à l'opérateur `?` et aux erreurs en général ; retiens pour l'instant qu'en Rust, une opération qui peut échouer le dit dans son type, et que le compilateur t'empêche d'utiliser le nombre sans avoir décidé quoi faire de l'erreur.

**Venant de Go :** `n, err := strconv.Atoi(s)` devient `s.parse::<i64>()`. La différence de fond : en Go, rien ne t'empêche d'utiliser `n` sans regarder `err`. En Rust, le nombre est *dans* le `Result`, et tu ne peux pas l'en sortir sans traiter l'erreur d'une façon ou d'une autre, même si cette façon est un `unwrap()` qui se voit dans le code.

### 2.11 Pour le labo

Le [labo 02](../labs/02-convertisseur/README.md) est un convertisseur : températures (`f64`), tailles en octets rendues lisibles (`1536` donne `1.5 Ko`, avec `as f64` et `{:.1}`), et durées écrites à la main (`1h30m`) analysées en `std::time::Duration`, avec une `String` que tu remplis caractère par caractère, un `parse::<u64>()`, un `checked_mul` contre le débordement et ton premier `Result<Duration, String>`. Et c'est le premier labo avec des tests : `cargo test` te dira si ta version est juste.

### À retenir

- `let` déclare une variable immuable, `let mut` une variable modifiable. L'immutabilité par défaut rend visibles les valeurs qui bougent, et prépare les emprunts du chapitre 4.
- Le shadowing (`let port = port.parse()...`) crée une nouvelle variable sous le même nom, éventuellement d'un autre type : idiomatique pour les étapes d'une même valeur.
- `const` (typée, en majuscules, recopiée à la compilation) dans 95 % des cas ; `static` pour une adresse unique.
- Entiers `i8` à `i128`, `u8` à `u128`, `usize` pour les indices et longueurs ; `i32` et `f64` par défaut ; `char` est un point de code Unicode de 4 octets.
- Le débordement panique en `dev` et s'enroule en `release` ; quand il est possible, dis ce que tu veux : `checked_`, `wrapping_`, `saturating_`.
- Aucune conversion implicite : `as` (brutal, tronque), `From`/`into` (sans perte), `TryFrom` (peut échouer, rend un `Result`).
- `String` possède son texte, `&str` est une vue empruntée ; paramètres en `&str`, champs et retours en `String`. Les deux sont de l'UTF-8 garanti.
- `len()` compte des octets, `chars()` des caractères ; `s[0]` ne compile pas, et `&s[a..b]` panique au milieu d'un caractère.
- `{}`, `{:?}`, `{:#?}`, `{x}`, `{:<10}`, `{:>8.2}` : la chaîne de format est vérifiée à la compilation.
- `"42".parse::<i32>()` rend un `Result` : le nombre n'en sort qu'après avoir décidé quoi faire de l'erreur.

---

← [1. Installer, outiller, lancer](01-installer-et-outiller.md) · [Sommaire](../README.md) · [3. Contrôle : if, loop, while, for, match](03-controle.md) →
