# 4. Possession et emprunts : le cœur de Rust

*Rust de zéro à la prod : chapitre 4 sur 16.* ← [3. Contrôle : if, loop, while, for, match](03-controle.md) · [Sommaire](../README.md) · [5. Collections et itérateurs](05-collections-et-iterateurs.md) →

Tout programme qui alloue de la mémoire doit répondre à une question : quand la libérer ? Le C te laisse répondre, avec `free`, et tu te trompes parfois : fuite, double libération, accès à une zone déjà rendue. Go et Python répondent à ta place, avec un ramasse-miettes qui passe régulièrement voir ce qui ne sert plus. Rust fait un troisième choix : il te demande de dire, dans le type de chaque variable et de chaque paramètre, **qui possède la valeur et qui ne fait que l'emprunter**, puis il vérifie à la compilation que ces déclarations sont cohérentes. Avec ces informations, le compilateur place lui-même la libération au bon endroit, et refuse tout programme qui pourrait lire une mémoire libérée ou la modifier pendant que quelqu'un d'autre la lit.

C'est le chapitre le plus important du cours, et celui qui fait râler tout débutant pendant une semaine. Les règles tiennent en quelques lignes ; ce qui demande du temps, c'est de lire les erreurs du compilateur et de comprendre ce qu'il protège. Ce chapitre montre chaque règle avec le programme qui la viole, l'erreur exacte, et la correction.

### 4.1 Trois règles

Toute la possession tient en trois phrases :

1. Chaque valeur a **un propriétaire**, une variable.
2. Il n'y a **qu'un seul propriétaire à la fois**.
3. Quand le propriétaire sort de sa portée, la valeur est **libérée**.

Et l'emprunt en ajoute deux :

4. On peut prêter une valeur sans la donner, par une **référence** : `&T` pour lire, `&mut T` pour modifier.
5. À un instant donné, une valeur a **soit plusieurs références `&T`, soit une seule `&mut T`**, jamais les deux. Et aucune référence ne peut vivre plus longtemps que la valeur qu'elle désigne.

Le reste du chapitre déroule ces cinq phrases.

### 4.2 Pile et tas

Pour comprendre ce que « posséder » veut dire, il faut savoir où vivent les valeurs. La **pile** contient les variables locales d'une fonction : leur taille est connue à la compilation, elles sont créées à l'entrée de la fonction et disparaissent à sa sortie, sans rien coûter. Le **tas** contient ce dont la taille n'est connue qu'à l'exécution, ou ce qui doit survivre à la fonction qui l'a créé : il faut demander la place à l'allocateur (le `malloc` du C) et la lui rendre (le `free`).

Un `i32`, un `f64`, un `bool`, un tableau `[u8; 4]`, un tuple de nombres vivent entièrement sur la pile. Une `String` et un `Vec` sont à cheval : une petite structure sur la pile, et leur contenu sur le tas.

```rust
use std::mem::size_of;

fn main() {
    let litteral: &str = "db01";
    let mut hote = String::from("db01");
    println!("{} {}", hote.len(), hote.capacity());
    hote.push_str(".prod.local");
    println!("{} {}", hote.len(), hote.capacity());
    let ports: Vec<u16> = vec![22, 80, 443];
    println!("{litteral} {hote} {ports:?}");
    println!(
        "String={} &str={} Vec={} u16={}",
        size_of::<String>(),
        size_of::<&str>(),
        size_of::<Vec<u16>>(),
        size_of::<u16>()
    );
}
```

```
$ cargo run
4 4
15 15
db01 db01.prod.local [22, 80, 443]
String=24 &str=16 Vec=24 u16=2
```

Une `String` fait 24 octets sur la pile, quelle que soit la longueur du texte : un pointeur vers les octets sur le tas, une longueur (les octets utilisés) et une capacité (les octets réservés). `push_str` a dû agrandir le bloc, d'où la nouvelle capacité. Un `Vec<T>` a exactement la même forme. Une `&str` fait 16 octets : un pointeur et une longueur, sans capacité, parce qu'elle ne possède rien et ne peut pas grandir. Le littéral `"db01"`, lui, est gravé dans le binaire ; `litteral` n'en est qu'une vue.

```
pile                                tas
hote : String                       +---+---+---+---+---+---+---+---+---
+-----------+                       | d | b | 0 | 1 | . | p | r | o | ...
| ptr       | --------------------> +---+---+---+---+---+---+---+---+---
| len = 15  |
| cap = 15  |
+-----------+
```

La question de la possession porte sur le bloc du tas : **quelqu'un doit le rendre à l'allocateur, une fois et une seule**, et personne ne doit le lire après. Pour un `i32` sur la pile, la question ne se pose pas.

### 4.3 Le déplacement

Que fait `let b = a;` quand `a` est une `String` ? Il copie les 24 octets de la pile : le pointeur, la longueur, la capacité. Le texte, lui, n'est pas copié. On a maintenant deux structures qui pointent vers le même bloc du tas. Si les deux restaient utilisables, qui libérerait le bloc ? Les deux, à la fin de la fonction : une double libération, le bug classique du C. Rust tranche : **après `let b = a;`, c'est `b` le propriétaire, et `a` n'est plus utilisable**. On dit que la valeur a été **déplacée** (*moved*).

```rust
fn main() {
    let a = String::from("db01");
    let b = a;
    println!("{a} {b}");
}
```

```
$ cargo build
error[E0382]: borrow of moved value: `a`
 --> src/main.rs:4:16
  |
2 |     let a = String::from("db01");
  |         - move occurs because `a` has type `String`, which does not implement the `Copy` trait
3 |     let b = a;
  |             - value moved here
4 |     println!("{a} {b}");
  |                ^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let b = a.clone();
  |              ++++++++
```

Lis le message en entier, c'est le plus important du cours. Trois positions : où `a` a été créé et pourquoi il se déplace (son type n'est pas `Copy`, on y vient), où il a été déplacé, et où on essaie de s'en servir après. Puis une suggestion. Le compilateur ne se contente pas de dire non : il raconte l'histoire de la variable.

```
avant : let b = a;                       après
a : String                               a : (inutilisable)
+-----------+      +---+---+---+---+     
| ptr       | ---> | d | b | 0 | 1 |     b : String
| len = 4   |      +---+---+---+---+     +-----------+      +---+---+---+---+
| cap = 4   |                            | ptr       | ---> | d | b | 0 | 1 |
+-----------+                            | len = 4   |      +---+---+---+---+
                                         | cap = 4   |
                                         +-----------+
```

Le déplacement ne coûte rien : 24 octets copiés, aucune allocation. Ce n'est pas une opération à l'exécution qui « invalide » `a` ; c'est le compilateur qui refuse de te laisser l'utiliser.

**Passer une valeur à une fonction la déplace aussi**, exactement comme une affectation :

```rust
fn afficher(s: String) {
    println!("j'ai reçu {s}");
}

fn main() {
    let hote = String::from("db01");
    afficher(hote);
    println!("{hote}");
}
```

```
$ cargo build
error[E0382]: borrow of moved value: `hote`
 --> src/main.rs:8:16
  |
6 |     let hote = String::from("db01");
  |         ---- move occurs because `hote` has type `String`, which does not implement the `Copy` trait
7 |     afficher(hote);
  |              ---- value moved here
8 |     println!("{hote}");
  |                ^^^^ value borrowed here after move
  |
note: consider changing this parameter type in function `afficher` to borrow instead if owning the value isn't necessary
 --> src/main.rs:1:16
  |
1 | fn afficher(s: String) {
  |    --------    ^^^^^^ this parameter takes ownership of the value
  |    |
  |    in this function
```

`afficher` a reçu la `String` : elle en est propriétaire, et la libère en se terminant. Après l'appel, il n'y a plus rien à afficher. Remarque la note : le compilateur suggère cette fois une meilleure correction que `clone`, changer le paramètre pour qu'il **emprunte** au lieu de prendre. C'est la bonne réponse, et le sujet de 4.5. Une fonction qui rend une valeur, à l'inverse, en donne la possession à l'appelant : `String::from` crée une `String` et te la donne.

**Venant de Python :** en Python, tout est référence : `copie = hotes` ne copie pas la liste, il colle une deuxième étiquette sur le même objet, et un `copie.append("db01")` apparaît dans `hotes`. C'est la source des « alias surprises », dont le plus célèbre est l'argument par défaut mutable, partagé entre tous les appels :

```python
def ajouter(h, liste=[]):
    liste.append(h)
    return liste
print(ajouter("a"), ajouter("b"))   # ['a', 'b'] ['a', 'b']
```

En Rust, `let copie = hotes;` déplace : il n'y a jamais deux noms qui peuvent modifier la même valeur sans que le type le dise. Un alias se déclare (`&` ou `&mut`), et les règles de 4.6 l'encadrent.

### 4.4 `Copy` et `Clone`

Tous les types ne se déplacent pas. Un entier se **copie** :

```rust
fn main() {
    let x = 5;
    let y = x;
    println!("{x} {y}");

    let a = String::from("db01");
    let b = a.clone();
    println!("{a} {b}");

    let cible = ("db01", 5432);
    let copie = cible;
    println!("{cible:?} {copie:?}");
}
```

```
$ cargo run
5 5
db01 db01
("db01", 5432) ("db01", 5432)
```

Un type est **`Copy`** quand le copier octet par octet suffit à faire une copie indépendante, parce qu'il ne possède rien sur le tas. Les entiers, les flottants, `bool`, `char`, les références partagées `&T`, et les tuples et tableaux qui ne contiennent que des types `Copy` le sont. Pour eux, `let y = x;` copie, et `x` reste utilisable. Le tuple `("db01", 5432)` est `Copy` : une `&str` et un `i32`, aucun des deux ne possède de mémoire.

Une `String` ne peut pas être `Copy` : copier ses 24 octets donnerait deux propriétaires du même bloc. Pour obtenir une vraie copie, tu le demandes avec **`.clone()`**, qui alloue un nouveau bloc sur le tas et y recopie le texte. C'est explicite parce que ça coûte : une allocation et une copie, proportionnelles à la taille. La règle de conception est là : **ce qui est bon marché est implicite (`Copy`), ce qui coûte s'écrit (`clone`)**. Quand tu lis `.clone()` dans du code Rust, tu sais qu'une copie a lieu ; quand tu ne le lis pas, tu sais qu'il n'y en a pas.

Tes propres types ne sont ni `Copy` ni `Clone` par défaut. Tu les rends tels avec `#[derive(Clone, Copy)]` (chapitre 6), et le compilateur refuse `Copy` sur un type qui contient une `String` ou un `Vec`. Le labo 03 a déjà `#[derive(Clone, Copy)]` sur son `enum Verdict` ; l'`Article` du labo 04, qui contient une `String`, n'a que `Clone`.

**Venant de Go :** en Go, `b := a` copie toujours la valeur : un `int`, une `struct` en entier, ou l'en-tête d'un slice (qui partage alors son tableau, le piège du chapitre 4 du cours Go). Rust distingue ce qui se copie sans risque (`Copy`) de ce qui se déplace, et ne te laisse jamais avec deux en-têtes qui pointent sur les mêmes données sans le savoir.

### 4.5 Prendre ou emprunter : lire une signature

Déplacer une valeur dans chaque fonction qu'on appelle serait invivable. La plupart des fonctions n'ont besoin que de **regarder** une valeur, ou de la **modifier** sur place. Pour ça, on leur prête une **référence** : `&x` crée une référence partagée (lecture seule), `&mut x` une référence mutable (lecture et écriture). Prêter s'appelle **emprunter**, et l'emprunt ne change pas le propriétaire.

Trois façons de passer un paramètre, trois contrats :

```rust
// Prend possession : l'appelant ne l'a plus.
fn archiver(journal: Vec<String>) -> usize {
    journal.len()
}

// Emprunte en lecture : l'appelant garde tout.
fn compter_erreurs(journal: &[String]) -> usize {
    let mut n = 0;
    for ligne in journal {
        if ligne.starts_with("ERROR") {
            n += 1;
        }
    }
    n
}

// Emprunte en écriture : modifie sur place.
fn ajouter(journal: &mut Vec<String>, ligne: &str) {
    journal.push(ligne.to_string());
}

fn main() {
    let mut journal = Vec::new();
    ajouter(&mut journal, "INFO démarrage");
    ajouter(&mut journal, "ERROR disque plein");
    ajouter(&mut journal, "ERROR disque toujours plein");
    println!("{} erreurs", compter_erreurs(&journal));
    println!("{} lignes", journal.len());
    let n = archiver(journal);
    println!("{n} lignes archivées");
}
```

```
$ cargo run
2 erreurs
3 lignes
3 lignes archivées
```

| Paramètre | Ce que dit la signature | L'appelant écrit | Après l'appel |
|---|---|---|---|
| `journal: Vec<String>` | je prends, j'en fais ce que je veux, je le libère | `archiver(journal)` | `journal` n'existe plus |
| `journal: &[String]` | je regarde, je ne touche à rien | `compter_erreurs(&journal)` | inchangé, toujours à l'appelant |
| `journal: &mut Vec<String>` | je modifie sur place | `ajouter(&mut journal, ...)` | modifié, toujours à l'appelant |

L'appelant écrit `&` ou `&mut` à l'appel : en lisant `ajouter(&mut journal, ...)`, tu sais sans ouvrir la fonction que `journal` va être modifié. Et une référence mutable ne s'obtient que sur une variable déclarée `mut` : la mutabilité se décide chez le propriétaire.

Comment choisir ? Dans cet ordre :

- **`&T`** par défaut : la fonction lit. C'est le cas de loin le plus fréquent. Et préfère la forme tranche, `&str` plutôt que `&String`, `&[T]` plutôt que `&Vec<T>` (4.8), qui accepte plus de choses sans rien coûter.
- **`&mut T`** si la fonction doit modifier la valeur de l'appelant.
- **`T`** si la fonction doit garder la valeur au-delà de l'appel (la ranger dans une structure, l'envoyer à un autre thread), ou la consommer (`archiver`, `fusionner` dans le labo). Dans ce cas, prendre la possession évite une copie : l'appelant qui n'en a plus besoin la donne, celui qui en a encore besoin fait un `clone` explicite.

Ce tableau est toute la grammaire des signatures Rust. Une fois que tu sais le lire, une signature te dit ce que la fonction fera de tes données, et c'est vérifié.

**Venant du C :** `&T` est un `const T *` qui ne peut pas être nul et ne peut pas survivre à la valeur pointée ; `&mut T` est un `T *` exclusif. On ne les déréférence presque jamais à la main : `journal.len()` marche sur une référence comme sur la valeur, le compilateur insère le `*` pour toi. Et il n'y a pas d'arithmétique de pointeurs.

### 4.6 La règle : plusieurs lecteurs ou un seul écrivain

Voici la règle qui fait de Rust ce qu'il est. À tout instant, pour une valeur donnée, tu peux avoir :

- **autant de références partagées `&T` que tu veux**, et personne ne modifie la valeur ;
- **ou une seule référence mutable `&mut T`**, et personne d'autre ne la lit ni ne la modifie.

Jamais les deux à la fois. Deux écrivains :

```rust
fn main() {
    let mut s = String::from("db01");
    let r1 = &mut s;
    let r2 = &mut s;
    r1.push('!');
    r2.push('?');
    println!("{s}");
}
```

```
$ cargo build
error[E0499]: cannot borrow `s` as mutable more than once at a time
 --> src/main.rs:4:14
  |
3 |     let r1 = &mut s;
  |              ------ first mutable borrow occurs here
4 |     let r2 = &mut s;
  |              ^^^^^^ second mutable borrow occurs here
5 |     r1.push('!');
  |     -- first borrow later used here
```

Un lecteur et un écrivain :

```rust
fn main() {
    let mut ports = vec![22, 80, 443];
    let premier = &ports[0];
    ports.push(8080);
    println!("premier : {premier}");
}
```

```
$ cargo build
error[E0502]: cannot borrow `ports` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let premier = &ports[0];
  |                    ----- immutable borrow occurs here
4 |     ports.push(8080);
  |     ^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("premier : {premier}");
  |                          ------- immutable borrow later used here
```

Ce second exemple montre pourquoi la règle existe, et ce n'est pas de la bureaucratie. `premier` pointe dans le bloc du tas qui contient les entiers. `push` peut avoir besoin d'agrandir ce bloc : il en alloue un plus grand, recopie les éléments, et **libère l'ancien**. `premier` pointerait alors dans de la mémoire rendue à l'allocateur. En C, ce programme compile, marche la plupart du temps, et affiche n'importe quoi le jour où le `realloc` déplace le bloc. En Go, le ramasse-miettes garde l'ancien tableau vivant, et `premier` lit une valeur périmée sans que personne ne le sache. En Rust, il ne compile pas.

Remarque les trois positions de chaque message : où l'emprunt commence, où on essaie le second, et **où le premier est encore utilisé après**. Un emprunt ne dure pas jusqu'à la fin du bloc : il dure **jusqu'à sa dernière utilisation**. Il suffit donc souvent de réordonner :

```rust
fn main() {
    let mut ports = vec![22, 80, 443];
    let premier = &ports[0];
    println!("premier : {premier}");
    ports.push(8080);
    let a = &ports;
    let b = &ports;
    println!("{} {}", a.len(), b.len());
    let m = &mut ports;
    m.push(9090);
    println!("{ports:?}");
}
```

```
$ cargo run
premier : 22
4 4
[22, 80, 443, 8080, 9090]
```

`premier` n'est plus utilisé après le premier `println!`, son emprunt est fini, et `push` passe. `a` et `b` lisent en même temps, sans souci. `m` emprunte en écriture, et son emprunt se termine après `m.push` : le dernier `println!` peut relire `ports`.

La même règle, appliquée aux threads, interdit les courses de données : deux threads qui écrivent la même valeur, ou l'un qui écrit pendant que l'autre lit, c'est exactement « deux `&mut` » ou « un `&` et un `&mut` ». Le chapitre 11 montre que Rust refuse de les compiler, là où Go les détecte à l'exécution, avec `-race`, si le test les déclenche.

**Venant de Go :** le ramasse-miettes te dispense de dire qui possède quoi, et c'est confortable. Rust t'oblige à le dire, et c'est plus de travail au début. En échange : pas de `nil` (une référence pointe toujours vers une valeur vivante), pas de course de données (la règle ci-dessus, vérifiée à la compilation), pas de pause du ramasse-miettes ni de mémoire gardée vivante par une sous-slice oubliée, et une libération à un instant connu, déterministe. C'est ce qui permet à Rust de tourner là où Go ne peut pas : noyau, firmware, bibliothèque appelée depuis du C ou du Python.

### 4.7 Portées et `Drop` : le `free` que le compilateur écrit pour toi

Troisième règle : quand le propriétaire sort de sa portée, la valeur est libérée. Concrètement, le compilateur insère, à chaque accolade fermante, l'appel au code de libération de chaque variable encore propriétaire de sa valeur, **dans l'ordre inverse de leur création**. Ce code s'appelle `drop`, et un type peut fournir le sien en implémentant le trait `Drop` (les traits sont au chapitre 7 ; lis cet exemple pour ce qu'il affiche) :

```rust
struct Connexion {
    nom: String,
}

impl Drop for Connexion {
    fn drop(&mut self) {
        println!("fermeture de {}", self.nom);
    }
}

fn ouvrir(nom: &str) -> Connexion {
    println!("ouverture de {nom}");
    Connexion {
        nom: nom.to_string(),
    }
}

fn main() {
    let _a = ouvrir("db01");
    {
        let _b = ouvrir("cache01");
        println!("fin du bloc");
    }
    let c = ouvrir("web01");
    drop(c);
    let _d = ouvrir("db02");
    println!("fin de main");
}
```

```
$ cargo run
ouverture de db01
ouverture de cache01
fin du bloc
fermeture de cache01
ouverture de web01
fermeture de web01
ouverture de db02
fin de main
fermeture de db02
fermeture de db01
```

`_b` est fermé à la fin de son bloc. `c` est fermé tout de suite par `drop(c)`, qui n'a rien de magique : c'est une fonction qui prend la possession de son argument et ne fait rien, donc la valeur meurt en sortant de `drop`. `_d` et `_a` sont fermés à la fin de `main`, le dernier créé en premier. Une valeur déplacée n'est libérée qu'une fois, par son dernier propriétaire : c'est pour ça que le déplacement existe.

Ce mécanisme porte un nom venu du C++, **RAII** (*Resource Acquisition Is Initialization*) : une ressource est liée à la vie d'une valeur. Il ne sert pas qu'à la mémoire. Un `File` ferme son descripteur quand il sort de portée, un verrou de `Mutex` se relâche, une connexion à une base retourne dans son pool, un dossier temporaire s'efface. Tu n'écris jamais `close()`, `unlock()` ni `free()`, et tu ne peux pas les oublier, même sur un chemin d'erreur ou un `return` anticipé.

**Venant du C :** c'est le `free` que tu plaçais à la main avant chaque `return` et dans chaque `goto cleanup`. Les trois bugs mémoire classiques du C deviennent des erreurs de compilation :

```c
char *hote = malloc(16);
strcpy(hote, "db01");
char *alias = hote;
free(hote);
printf("%s\n", alias);   /* use after free */
free(alias);             /* double free */
```

L'oubli de `free` (la fuite) disparaît parce que le compilateur l'écrit. Le double `free` disparaît parce qu'il n'y a qu'un propriétaire. L'utilisation après `free` disparaît parce qu'une référence ne peut pas survivre à sa valeur (4.9) et qu'une valeur déplacée n'est plus accessible. Ce n'est pas un analyseur statique qui t'avertit : c'est le langage qui refuse le programme.

**Venant de Go :** `defer f.Close()` devient inutile, `Drop` le fait. Et contrairement à un `defer`, qui s'exécute à la fin de la *fonction*, `drop` s'exécute à la fin du *bloc* : un fichier ouvert dans une boucle est fermé à chaque tour, pas à la fin de la fonction après mille tours.

### 4.8 Les tranches : `&str` et `&[T]`, des vues empruntées

Une **tranche** est une référence vers une suite contiguë d'éléments qui appartiennent à quelqu'un d'autre : un pointeur et une longueur. `&str` est une tranche de texte, `&[T]` une tranche d'éléments de type `T`. Tu les as croisées au chapitre 2 (`&s[2..]`, `trim`, `split`) ; voici ce qu'elles sont vraiment.

```rust
fn premier_mot(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

fn somme(valeurs: &[u64]) -> u64 {
    let mut total = 0;
    for v in valeurs {
        total += v;
    }
    total
}

fn main() {
    let ligne = String::from("ERROR disque plein");
    let niveau = premier_mot(&ligne);
    println!("{niveau}");
    println!("{}", premier_mot("INFO"));

    let tailles = vec![512, 1024, 2048, 4096];
    println!("{}", somme(&tailles));
    println!("{}", somme(&tailles[1..3]));
    println!("{}", somme(&[1, 2, 3]));
}
```

```
$ cargo run
ERROR
INFO
7680
3072
6
```

`premier_mot` rend un morceau de son argument, sans rien copier : `niveau` pointe dans le texte de `ligne`. Elle accepte une `String` empruntée (`&ligne` se convertit seul en `&str`) comme un littéral. `somme` accepte un `Vec` entier, un morceau de `Vec`, ou un tableau. C'est pourquoi un paramètre en lecture s'écrit `&str` et `&[T]`, jamais `&String` ni `&Vec<T>` : la forme tranche accepte tout, et clippy te reprend sinon (`ptr_arg`).

Une tranche est un emprunt, avec ses règles. Tant que `niveau` sert, `ligne` ne peut pas être modifiée (même `premier_mot` au-dessus) :

```rust
fn main() {
    let mut ligne = String::from("ERROR disque plein");
    let niveau = premier_mot(&ligne);
    ligne.clear();
    println!("{niveau}");
}
```

```
$ cargo build
error[E0502]: cannot borrow `ligne` as mutable because it is also borrowed as immutable
  --> src/main.rs:11:5
   |
10 |     let niveau = premier_mot(&ligne);
   |                              ------ immutable borrow occurs here
11 |     ligne.clear();
   |     ^^^^^^^^^^^^^ mutable borrow occurs here
12 |     println!("{niveau}");
   |                ------ immutable borrow later used here
```

En C, `niveau` serait un `char *` dans un tampon que `clear` vient de vider, et tu lirais des octets périmés. Ici, le compilateur sait que `niveau` vient de `ligne`. Comment le sait-il ? Par la signature de `premier_mot`, et c'est le sujet de la section suivante.

**Venant de Go :** `&[T]` est le slice de Go sans la capacité : on ne peut pas faire `append` sur une tranche, seulement lire (ou modifier les éléments en place avec `&mut [T]`). Le piège n°1 de Go, l'`append` sur une sous-slice qui écrase l'original, n'existe pas : pour faire grandir, il faut le `Vec` lui-même, en `&mut Vec<T>`.

### 4.9 Les durées de vie

Une **durée de vie** (*lifetime*) est la portion du programme pendant laquelle une référence est valable. Le compilateur la calcule pour chaque référence, et vérifie qu'aucune ne survit à la valeur qu'elle désigne. Dans le corps d'une fonction, il voit tout et n'a besoin de rien. À la frontière d'une fonction, il ne regarde que la signature, et c'est là que tu dois parfois l'aider :

```rust
fn plus_long(a: &str, b: &str) -> &str {
    if a.len() >= b.len() { a } else { b }
}
```

```
$ cargo build
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:35
  |
1 | fn plus_long(a: &str, b: &str) -> &str {
  |                 ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
1 | fn plus_long<'a>(a: &'a str, b: &'a str) -> &'a str {
  |             ++++     ++          ++          ++
```

Le message dit tout : la fonction rend une référence, mais la signature ne dit pas si elle vient de `a` ou de `b`. Pour l'appelant, ça change tout : doit-il garder `a` vivant, `b`, ou les deux ? On l'écrit avec un **paramètre de durée de vie**, un nom qui commence par une apostrophe, déclaré entre chevrons comme un type générique :

```rust
fn plus_long<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    let a = String::from("db01.prod");
    let resultat;
    {
        let b = String::from("cache01");
        resultat = plus_long(&a, &b);
        println!("{resultat}");
    }
}
```

```
$ cargo run
db01.prod
```

`'a` se lit « une certaine durée ». La signature dit : « donne-moi deux références valables au moins pendant une même durée `'a`, et je te rends une référence valable pendant cette durée ». En pratique, `'a` est la plus courte des deux. **Une durée de vie ne change rien à l'exécution et ne prolonge la vie de rien** : c'est une contrainte que le compilateur vérifie chez l'appelant. Déplace le `println!` hors du bloc, après la mort de `b` :

```
$ cargo build
error[E0597]: `b` does not live long enough
  --> src/main.rs:10:34
   |
 9 |         let b = String::from("cache01");
   |             - binding `b` declared here
10 |         resultat = plus_long(&a, &b);
   |                                  ^^ borrowed value does not live long enough
11 |     }
   |     - `b` dropped here while still borrowed
12 |     println!("{resultat}");
   |                -------- borrow later used here
```

Ici `resultat` vaut `a`, qui est encore vivant, et le programme serait correct à l'exécution. Mais le compilateur ne raisonne que sur la signature, qui dit « peut-être `b` ». Il refuse ce qui *pourrait* être faux. C'est le prix des garanties à la compilation, et la raison de bien choisir ses signatures. Dans le labo 04, `chercher<'a>(inv: &'a [Article], nom: &str) -> Option<&'a Article>` ne met `'a` que sur l'inventaire : le résultat vient de lui, et le nom cherché peut mourir juste après l'appel.

Le cas le plus courant de référence qui ne vivrait pas assez longtemps est la fonction qui rend une référence vers sa propre variable locale :

```rust
fn nom_complet(hote: &str) -> &str {
    let s = format!("{hote}.prod.local");
    &s
}
```

```
$ cargo build
error[E0515]: cannot return reference to local variable `s`
 --> src/main.rs:3:5
  |
3 |     &s
  |     ^^ returns a reference to data owned by the current function
```

`s` est libérée à la fin de la fonction, une référence vers elle serait pendante. C'est le « retourner l'adresse d'une variable locale » du C, qui compile en C avec un simple avertissement. La correction n'est pas une annotation : c'est de **rendre la valeur possédée**, `fn nom_complet(hote: &str) -> String`, et de laisser l'appelant en devenir propriétaire. Retiens le réflexe : une fonction qui fabrique une valeur la rend possédée ; une fonction qui rend une référence la prend forcément dans l'un de ses paramètres.

**Les règles d'élision.** Si tu as écrit des fonctions qui prennent et rendent des `&str` sans jamais écrire `'a`, c'est que le compilateur les déduit dans les cas évidents, avec trois règles :

1. Chaque référence en paramètre reçoit sa propre durée de vie.
2. S'il n'y a **qu'une seule** référence en paramètre, sa durée de vie est donnée à toutes les références du résultat. C'est le cas de `premier_mot(s: &str) -> &str` : le résultat vient forcément de `s`.
3. Si c'est une méthode qui prend `&self` ou `&mut self` (chapitre 6), la durée de vie de `self` va au résultat.

Si les règles ne suffisent pas à attribuer une durée à chaque référence du résultat, c'est l'erreur `E0106`, et tu écris les `'a`. Avec deux paramètres références, comme `plus_long`, la règle 2 ne s'applique pas.

**`'static`** est la durée de vie de ce qui vit aussi longtemps que le programme. Les chaînes littérales sont gravées dans le binaire, leur type complet est `&'static str` :

```rust
fn niveau(code: u8) -> &'static str {
    match code {
        0 => "debug",
        1 => "info",
        _ => "erreur",
    }
}

fn nom_complet(hote: &str) -> String {
    format!("{hote}.prod.local")
}

fn main() {
    println!("{} {}", niveau(1), nom_complet("db01"));
}
```

```
$ cargo run
info db01.prod.local
```

`niveau` peut rendre une référence sans paramètre, parce que ses chaînes ne meurent jamais. **Piège :** quand le compilateur te suggère d'ajouter `'static` pour faire taire une erreur de durée de vie, c'est presque toujours la mauvaise correction. Ça ne marche que pour des données qui vivent vraiment tout le programme ; pour le reste, la solution est de rendre une valeur possédée.

Une structure peut aussi contenir des références, et doit alors déclarer leur durée de vie : `struct Ligne<'a> { niveau: &'a str, message: &'a str }` est une ligne de journal découpée sans copie, qui ne peut pas survivre au texte d'où elle vient. C'est puissant pour les analyseurs rapides, et contraignant partout ailleurs. **Conseil de débutant : dans tes structures, stocke des `String` et des `Vec`, pas des références.** Tu verras les durées de vie plus en détail au chapitre 10.

### 4.10 Quand on se bat avec le vérificateur

Le vérificateur d'emprunts (*borrow checker*) aura raison dans 95 % des cas, et tu finiras par écrire spontanément du code qu'il accepte. En attendant, voici les situations où tout le monde coince, et les sorties.

**Cloner quand c'est bon marché.** Si tu as besoin d'une valeur pendant que tu modifies son propriétaire, une copie coupe le lien :

```rust
fn main() {
    let mut hotes = vec![String::from("web01"), String::from("db01")];
    let premier = hotes[0].clone();
    hotes.push(String::from("cache01"));
    println!("ajouté après {premier}");

    let mut ports = vec![22, 80, 443];
    let max = *ports.iter().max().unwrap_or(&0);
    ports.push(max + 1);
    println!("{ports:?}");
}
```

```
$ cargo run
ajouté après web01
[22, 80, 443, 444]
```

`premier` est une `String` à lui, plus un emprunt de `hotes`. Pour un entier, c'est encore plus simple : `*` déréférence et copie la valeur, et l'emprunt s'arrête là. Cloner une chaîne de dix octets pour avoir un code clair est un excellent marché ; cloner un `Vec` d'un million d'éléments dans une boucle ne l'est pas. Ne te sens pas coupable d'un `clone` sur un petit texte, mais ne t'en sers pas pour éviter de comprendre une erreur.

**Restructurer.** Un emprunt dure jusqu'à sa dernière utilisation : lis ce dont tu as besoin, finis-en avec la référence, puis modifie. La plupart des `E0502` se corrigent en déplaçant une ligne, comme en 4.6.

**Des indices plutôt que des références.** Une référence dans une collection bloque toute la collection ; un indice (`usize`) n'emprunte rien. Pour trouver un élément puis supprimer, `position()` rend un indice, et `remove(i)` passe. C'est ce que fait `retirer` dans le labo 04. Les structures complexes (graphes, arbres avec parents) se construisent d'ailleurs souvent comme un `Vec` de nœuds reliés par des indices.

**Deux `&mut` dans la même collection.** Le compilateur ne sait pas que `charges[0]` et `charges[2]` sont deux cases différentes : pour lui, ce sont deux emprunts mutables de `charges`.

```rust
fn main() {
    let mut charges = [10, 20, 30];
    let a = &mut charges[0];
    let b = &mut charges[2];
    std::mem::swap(a, b);
    println!("{charges:?}");
}
```

```
$ cargo build
error[E0499]: cannot borrow `charges[_]` as mutable more than once at a time
 --> src/main.rs:4:13
  |
3 |     let a = &mut charges[0];
  |             --------------- first mutable borrow occurs here
4 |     let b = &mut charges[2];
  |             ^^^^^^^^^^^^^^^ second mutable borrow occurs here
5 |     std::mem::swap(a, b);
  |                    - first borrow later used here
  |
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

La réponse est dans l'aide : `split_at_mut` coupe une tranche en deux tranches mutables qui ne se chevauchent pas, ce que la bibliothèque standard garantit. Et pour ce cas précis, il existe une méthode toute faite :

```rust
fn main() {
    let mut charges = [10, 20, 30];
    let (gauche, droite) = charges.split_at_mut(1);
    std::mem::swap(&mut gauche[0], &mut droite[1]);
    println!("{charges:?}");
    charges.swap(0, 1);
    println!("{charges:?}");
}
```

```
$ cargo run
[30, 20, 10]
[20, 30, 10]
```

**Chercher puis insérer dans une table : `entry`.** Le réflexe venu de Go, « je regarde si la clé existe, sinon j'insère, puis je modifie », enchaîne un emprunt en lecture et un en écriture. L'API `entry` des `HashMap` fait tout en une opération, avec un seul emprunt :

```rust
use std::collections::HashMap;

fn main() {
    let lignes = ["web01 GET", "db01 SELECT", "web01 POST", "web01 GET"];
    let mut par_hote: HashMap<&str, u32> = HashMap::new();
    for ligne in lignes {
        let hote = ligne.split(' ').next().unwrap_or("");
        *par_hote.entry(hote).or_insert(0) += 1;
    }
    let mut hotes: Vec<_> = par_hote.into_iter().collect();
    hotes.sort();
    println!("{hotes:?}");
}
```

```
$ cargo run
[("db01", 1), ("web01", 3)]
```

`entry(hote)` rend un emplacement, occupé ou libre ; `or_insert(0)` y met 0 s'il est libre et rend une `&mut u32` vers la valeur ; `*... += 1` l'incrémente. C'est le `compteur[h]++` de Go, en une ligne aussi. Le chapitre 5 fait le tour des `HashMap`.

### 4.11 Modifier une collection pendant qu'on la parcourt

C'est l'erreur que tout le monde fait, dans tous les langages. En Python, modifier une liste pendant un `for` saute des éléments ou boucle sans fin, et modifier un dictionnaire lève `RuntimeError: dictionary changed size during iteration`. En Go, ça compile, et `range` parcourt la version d'avant sans voir les ajouts : c'est ce que tu voulais, ou un bug, selon le jour. En Rust :

```rust
fn main() {
    let mut hotes = vec![String::from("web01"), String::from("db01")];
    for h in &hotes {
        if h.starts_with("web") {
            hotes.push(format!("{h}-bis"));
        }
    }
    println!("{hotes:?}");
}
```

```
$ cargo build
error[E0502]: cannot borrow `hotes` as mutable because it is also borrowed as immutable
 --> src/main.rs:5:13
  |
3 |     for h in &hotes {
  |              ------
  |              |
  |              immutable borrow occurs here
  |              immutable borrow later used here
4 |         if h.starts_with("web") {
5 |             hotes.push(format!("{h}-bis"));
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

La boucle emprunte `hotes` en lecture pendant toute sa durée, et `push` voudrait l'emprunter en écriture au milieu. C'est exactement le cas de 4.6 : `push` peut réallouer le bloc que la boucle est en train de parcourir. Les bonnes façons, selon ce que tu veux faire :

```rust
fn main() {
    let mut hotes = vec![
        String::from("web01"),
        String::from("db01"),
        String::from("web02"),
    ];

    for h in hotes.iter_mut() {
        h.push_str(".local");
    }
    println!("{hotes:?}");

    hotes.retain(|h| !h.starts_with("db"));
    println!("{hotes:?}");

    let mut nouveaux = Vec::new();
    for h in &hotes {
        nouveaux.push(h.replace(".local", "-bis.local"));
    }
    hotes.extend(nouveaux);
    println!("{hotes:?}");

    for i in 0..hotes.len() {
        if hotes[i].contains("bis") {
            let copie = hotes[i].to_uppercase();
            hotes.push(copie);
        }
    }
    println!("{}", hotes.len());
}
```

```
$ cargo run
["web01.local", "db01.local", "web02.local"]
["web01.local", "web02.local"]
["web01.local", "web02.local", "web01-bis.local", "web02-bis.local"]
6
```

- **Modifier chaque élément sur place** : `iter_mut()` donne une `&mut` vers chaque élément, à tour de rôle. Le `Vec` ne change pas de taille, rien n'est réalloué.
- **Supprimer des éléments** : `retain` garde ceux pour lesquels la fonction rend `true`, en une passe. C'est le `filter` en place.
- **Ajouter des éléments** : collecte les ajouts dans un second `Vec` pendant la boucle, et ajoute-les après avec `extend`. Deux phases, deux emprunts qui ne se chevauchent pas.
- **Boucle par indices**, en dernier recours : `0..hotes.len()` est calculé une seule fois, avant la boucle, et chaque `hotes[i]` est un emprunt d'une ligne. Le `to_uppercase()` fabrique une `String` possédée avant le `push`, donc aucun emprunt ne traverse l'ajout. Les éléments ajoutés ne sont pas parcourus : pas de boucle infinie.

Même logique pour une boucle qui consomme : `for h in hotes` (sans `&`) déplace `hotes` dans la boucle, et le compilateur le dit si tu t'en sers après : `` `hotes` moved due to this implicit call to `.into_iter()` ``, avec la suggestion `for h in &hotes`. Le chapitre 5 montre les itérateurs (`map`, `filter`, `collect`) qui remplacent la plupart de ces boucles par une expression.

### 4.12 Pour le labo

Le [labo 04](../labs/04-inventaire/README.md) est un inventaire d'articles, un `Vec<Article>` où chaque article possède son nom. Le code des fonctions est court, et les signatures sont données : `&[Article]` pour lire, `&mut Vec<Article>` pour modifier, `Vec<Article>` pour prendre, `<'a>` quand deux références arrivent et qu'une repart. Les `TODO` te font écrire le corps, puis provoquer exprès `E0106`, `E0382`, `E0499` et `E0502` pour les lire en vrai, et la solution explique pourquoi chaque emprunt est ce qu'il est, y compris les durées de vie élidées de `noms`.

### À retenir

- Chaque valeur a un seul propriétaire ; quand il sort de portée, la valeur est libérée (`Drop`), dans l'ordre inverse de création. Le compilateur écrit les `free`, et RAII ferme aussi fichiers, verrous et connexions.
- `let b = a;` et le passage en paramètre **déplacent** une `String` ou un `Vec` : `a` est inutilisable ensuite (`E0382`). Les types `Copy` (entiers, `bool`, `char`, `&T`) se copient ; le reste se clone explicitement, avec `.clone()`.
- Une signature est un contrat : `T` prend, `&T` lit, `&mut T` modifie. Par défaut, emprunte en lecture, et préfère `&str` et `&[T]` à `&String` et `&Vec<T>`.
- La règle : plusieurs `&T` **ou** une seule `&mut T`, jamais les deux (`E0499`, `E0502`). Elle empêche les références pendantes après réallocation et, entre threads, les courses de données.
- Un emprunt dure jusqu'à sa dernière utilisation, pas jusqu'à la fin du bloc : réordonner suffit souvent.
- Les tranches `&str` et `&[T]` sont des vues empruntées, sans copie, et elles bloquent leur propriétaire tant qu'elles servent.
- Une durée de vie `'a` ne prolonge rien : elle dit d'où vient une référence rendue. Écris-la quand deux références entrent et qu'une sort (`E0106`) ; l'élision s'en charge quand il n'y en a qu'une.
- Ne rends jamais une référence vers une variable locale (`E0515`) : rends la valeur possédée. Et méfie-toi de `'static` suggéré pour faire taire une erreur.
- Pour s'en sortir : `clone` quand c'est bon marché, restructurer, des indices au lieu de références, `split_at_mut`, `entry`.
- Pas de `push` pendant un `for` sur la même collection : `iter_mut` pour modifier en place, `retain` pour supprimer, un second `Vec` puis `extend` pour ajouter.

---

← [3. Contrôle : if, loop, while, for, match](03-controle.md) · [Sommaire](../README.md) · [5. Collections et itérateurs](05-collections-et-iterateurs.md) →
