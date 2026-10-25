# 10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe

*Rust de zéro à la prod : chapitre 10 sur 16.* ← [9. Modules, crates, Cargo, tests, documentation](09-modules-crates-tests.md) · [Sommaire](../README.md) · [11. Concurrence : threads, channels, Arc et Mutex, Send et Sync](11-concurrence.md) →

Le chapitre 4 t'a donné les règles : un propriétaire, des emprunts partagés ou un emprunt exclusif, des références qui ne survivent pas à ce qu'elles désignent. Ce chapitre montre ce qu'il y a dessous, et ce qu'on fait quand les règles ne suffisent pas : quand une donnée doit avoir plusieurs propriétaires, quand on doit modifier à travers une référence partagée, quand une struct contient une référence, quand il faut appeler du C. Tu y verras aussi pourquoi les itérateurs ne coûtent rien, ce que coûte un `dyn`, pourquoi `cargo build` est lent, et ce que le mot `unsafe` autorise vraiment. Rien de tout ça n'est indispensable pour écrire un premier outil, mais tout l'est pour lire le code des autres, et pour choisir entre `Box`, `Rc` et une référence sans tirer à pile ou face.

### 10.1 Pile et tas, pour de vrai

Chaque thread a une **pile** : une zone contiguë, quelques méga-octets (8 Mio pour le thread principal sur Linux et macOS, 2 Mio pour les threads que Rust lance), où chaque appel de fonction réserve un cadre pour ses variables locales. Réserver, c'est déplacer un pointeur ; libérer, c'est le remettre. Rien n'est plus rapide. Le **tas** est le reste de la mémoire, géré par l'allocateur (`malloc` de la libc par défaut) : on y demande un bloc de n'importe quelle taille, qui vit aussi longtemps qu'on veut, au prix d'un appel à l'allocateur.

En Rust, **toute valeur vit d'abord sur la pile, ou dans le champ d'une autre valeur**. Le tas n'apparaît que par un type qui le demande : `Box`, `Vec`, `String`, `HashMap`, `Rc`. Et ces types sont eux-mêmes de petites valeurs de taille fixe, sur la pile, qui pointent vers leur tampon :

```rust
use std::mem::{align_of, size_of, size_of_val};

#[allow(dead_code)]
struct Mesure {
    actif: bool,
    valeur: u64,
    code: u16,
}

#[allow(dead_code)]
#[repr(C)]
struct MesureC {
    actif: bool,
    valeur: u64,
    code: u16,
}

fn main() {
    let n: u64 = 42;
    let tableau = [0u8; 1000];
    let texte = String::from("bonjour");
    let v = vec![1u32, 2, 3];

    println!("u64        : {} octets, aligné sur {}", size_of::<u64>(), align_of::<u64>());
    println!("[u8; 1000] : {} octets, sur la pile", size_of_val(&tableau));
    println!("String     : {} octets sur la pile, {} sur le tas", size_of_val(&texte), texte.capacity());
    println!("Vec<u32>   : {} octets sur la pile, {} sur le tas", size_of_val(&v), v.capacity() * size_of::<u32>());
    println!("Mesure     : {} octets (Rust réordonne les champs)", size_of::<Mesure>());
    println!("MesureC    : {} octets (#[repr(C)] garde l'ordre)", size_of::<MesureC>());
    println!("adresse de n       : {:p}", &n);
    println!("adresse de texte   : {:p}  (la String elle-même)", &texte);
    println!("adresse des octets : {:p}  (son tampon)", texte.as_ptr());
}
```

```
$ cargo run -q
u64        : 8 octets, aligné sur 8
[u8; 1000] : 1000 octets, sur la pile
String     : 24 octets sur la pile, 7 sur le tas
Vec<u32>   : 24 octets sur la pile, 12 sur le tas
Mesure     : 16 octets (Rust réordonne les champs)
MesureC    : 24 octets (#[repr(C)] garde l'ordre)
adresse de n       : 0x16d525c90
adresse de texte   : 0x16d526080  (la String elle-même)
adresse des octets : 0x103011a60  (son tampon)
```

Une `String` fait 24 octets quelle que soit sa longueur : un pointeur, une capacité, une longueur. Ses 7 octets de texte sont ailleurs, à une adresse très différente (la pile est en haut de l'espace d'adressage, le tas en bas). Un `[u8; 1000]` fait 1000 octets **en place** : un tableau de taille fixe est une valeur, pas un pointeur.

L'**alignement** explique les deux dernières tailles. Un `u64` doit commencer à une adresse multiple de 8. Dans l'ordre déclaré (`bool`, `u64`, `u16`), il faut 7 octets de remplissage après le `bool` et 6 à la fin : 24 octets. Rust se réserve le droit de **réordonner les champs** pour minimiser le remplissage (`u64`, `u16`, `bool`, plus 5 octets) : 16. `#[repr(C)]` interdit ce réordonnancement et donne exactement la disposition qu'un compilateur C donnerait : indispensable pour passer une struct à du C (10.12), inutile ailleurs.

`size_of` et `size_of_val` sont dans le prélude depuis Rust 1.80. Le labo te fait deviner la taille d'une vingtaine de types avant de les mesurer : fais-le vraiment.

**Venant du C :** c'est le même modèle, sans `malloc` ni `free` à écrire. Le `free` est le `Drop` du type, appelé automatiquement quand le propriétaire sort de la portée (chapitre 4). Un `Vec` est exactement le trio `(pointeur, capacité, longueur)` que tu écrirais en C, avec le `realloc` dedans.

**Venant de Go :** en Go, le compilateur décide par l'analyse d'échappement si `&s` va sur la pile ou sur le tas (`-gcflags=-m` te le dit). En Rust, **c'est toi qui décides, et ça se voit dans le type** : une valeur est sur la pile, un `Box<T>` sur le tas, et `return &locale` ne compile pas (`E0515`) au lieu de faire échapper la variable. Pas de ramasse-miettes : la mémoire est rendue à l'instant où le propriétaire disparaît. Donc pas de pause, pas de `GOGC` à régler, et une consommation mémoire qui suit les données au lieu de les doubler. Le prix : c'est toi qui choisis les propriétaires.

### 10.2 `Box<T>` : une valeur sur le tas, un seul propriétaire

`Box::new(v)` déplace `v` sur le tas et rend un pointeur unique vers elle. Quand la `Box` est détruite, la valeur l'est aussi. C'est `malloc` plus `free`, avec la possession du chapitre 4 par-dessus. On en a besoin dans trois cas, pas plus.

**Un type récursif.** Une expression arithmétique est un arbre : une somme contient deux expressions. Écrit naïvement :

```rust
#[allow(dead_code)]
enum Expr {
    Nombre(i64),
    Somme(Expr, Expr),
}

fn main() {}
```

```
$ cargo build
error[E0072]: recursive type `Expr` has infinite size
 --> src/bin/c10_rec.rs:2:1
  |
2 | enum Expr {
  | ^^^^^^^^^
3 |     Nombre(i64),
4 |     Somme(Expr, Expr),
  |           ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
4 |     Somme(Box<Expr>, Expr),
  |           ++++    +
```

Le compilateur doit connaître la taille d'un `Expr` pour le poser sur la pile, et un `Expr` qui contient un `Expr` serait infini. Une `Box` a une taille fixe (un pointeur) : la récursion passe par le tas.

**Des types différents derrière un même trait.** `Vec<Box<dyn Sonde>>` (chapitre 7) : chaque élément peut être un type concret différent, de taille différente, parce que le `Vec` ne stocke que des pointeurs.

**Une très grosse valeur** qu'on ne veut ni sur la pile (8 Mio en tout) ni copier à chaque déplacement : déplacer une `Box` copie 8 octets, pas la valeur.

```rust
// Une expression arithmétique : un arbre, donc un type récursif.
enum Expr {
    Nombre(i64),
    Somme(Box<Expr>, Box<Expr>),
    Produit(Box<Expr>, Box<Expr>),
}

fn evaluer(e: &Expr) -> i64 {
    match e {
        Expr::Nombre(n) => *n,
        Expr::Somme(a, b) => evaluer(a) + evaluer(b),
        Expr::Produit(a, b) => evaluer(a) * evaluer(b),
    }
}

trait Sonde {
    fn nom(&self) -> String;
}

struct Tcp(u16);
struct Http(&'static str);

impl Sonde for Tcp {
    fn nom(&self) -> String {
        format!("tcp/{}", self.0)
    }
}

impl Sonde for Http {
    fn nom(&self) -> String {
        format!("http {}", self.0)
    }
}

fn main() {
    // (2 + 3) * 4
    let e = Expr::Produit(
        Box::new(Expr::Somme(Box::new(Expr::Nombre(2)), Box::new(Expr::Nombre(3)))),
        Box::new(Expr::Nombre(4)),
    );
    println!("(2 + 3) * 4 = {}", evaluer(&e));

    // Des types différents dans un même Vec : Box<dyn Trait>.
    let sondes: Vec<Box<dyn Sonde>> = vec![Box::new(Tcp(5432)), Box::new(Http("/health"))];
    for s in &sondes {
        println!("{}", s.nom());
    }

    // 8 Mio sur le tas : la pile du thread principal n'en a que 8 en tout.
    let gros = vec![0u8; 8 * 1024 * 1024].into_boxed_slice();
    println!("{} octets sur le tas, {} sur la pile", gros.len(), size_of_val(&gros));
}
```

```
$ cargo run -q
(2 + 3) * 4 = 20
tcp/5432
http /health
8388608 octets sur le tas, 16 sur la pile
```

Note `evaluer(a)` où `a` est un `&Box<Expr>` : la conversion vers `&Expr` est automatique (*deref coercion*), une `Box` se manipule comme la valeur qu'elle contient. Et note les 16 octets du `Box<[u8]>` : pointeur plus longueur, un **gros pointeur**, comme `&str` et `&[T]`. Un `Box<dyn Sonde>` fait aussi 16 octets : pointeur vers la valeur, pointeur vers la table des méthodes (la *vtable*).

**La niche.** Un `Box` ne peut jamais être nul. Rust s'en sert : `Option<Box<T>>` représente `None` par le pointeur nul, et fait donc exactement la taille d'un `Box<T>`, sans étiquette. Même chose pour `Option<&T>`, `Option<String>` (dont le pointeur interne n'est jamais nul) ou `Option<char>` (0x110000 n'est pas un `char`). C'est le `NULL` du C, sans le risque de l'oublier : le compilateur t'oblige à traiter le `None`. Le labo le vérifie par un test.

### 10.3 `Rc<T>` : plusieurs propriétaires

Une `Box` a un seul propriétaire. Parfois la donnée en a naturellement plusieurs : une configuration partagée par trois services, un nœud de graphe pointé par deux autres, un cache vu de partout. Aucun n'est « le » propriétaire, et la donnée doit vivre tant que l'un d'eux l'utilise. C'est `Rc<T>` (*reference counted*) : la valeur sur le tas, accompagnée d'un compteur. `Rc::clone` ne copie pas la valeur, il incrémente le compteur et rend un second pointeur ; chaque `Drop` d'un `Rc` le décrémente ; à zéro, la valeur est libérée.

```rust
use std::rc::Rc;

#[derive(Debug)]
struct Config {
    dsn: String,
}

struct Service {
    nom: &'static str,
    config: Rc<Config>,
}

fn main() {
    let config = Rc::new(Config { dsn: "postgres://db1/app".into() });
    println!("propriétaires : {}", Rc::strong_count(&config));

    let api = Service { nom: "api", config: Rc::clone(&config) };
    let worker = Service { nom: "worker", config: Rc::clone(&config) };
    println!("propriétaires : {}", Rc::strong_count(&config));
    println!("{} et {} lisent {}", api.nom, worker.nom, worker.config.dsn);
    println!("même adresse : {}", Rc::ptr_eq(&api.config, &worker.config));

    drop(api);
    println!("après drop(api) : {}", Rc::strong_count(&config));
    drop(worker);
    drop(config); // le dernier : la Config est libérée ici
}
```

```
$ cargo run -q
propriétaires : 1
propriétaires : 3
api et worker lisent postgres://db1/app
même adresse : true
après drop(api) : 2
```

On écrit `Rc::clone(&config)` plutôt que `config.clone()` : les deux font la même chose, mais la première forme dit au lecteur que c'est un clone bon marché (un compteur), pas une copie de la config. C'est une convention, que clippy peut imposer avec le lint optionnel `clone_on_ref_ptr`.

Ce que `Rc` ne permet pas : **modifier**. Un `Rc<T>` donne des `&T`, jamais des `&mut T`, parce que plusieurs propriétaires pourraient sinon modifier en même temps ce que d'autres lisent :

```rust
use std::rc::Rc;

fn main() {
    let liste = Rc::new(vec![1, 2, 3]);
    let autre = Rc::clone(&liste);
    liste.push(4);
    println!("{autre:?}");
}
```

```
$ cargo build
error[E0596]: cannot borrow data in an `Rc` as mutable
 --> src/bin/c10_rcmut.rs:6:5
  |
6 |     liste.push(4);
  |     ^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Vec<i32>>`
```

La réponse est la section suivante. Et `Rc` est réservé à **un seul thread** : son compteur n'est pas atomique, le chapitre 11 montre l'erreur si on essaie, et son grand frère `Arc`.

**Venant de Python :** CPython fait exactement ça pour **chaque** objet : un compteur de références dans l'en-tête, incrémenté à chaque affectation, et `sys.getrefcount(x)` le montre (avec un de plus, pour l'argument de l'appel). `Rc::strong_count` est le même chiffre. La différence est que Rust ne le fait que là où tu l'as demandé, en écrivant `Rc`, et que tout le reste est libéré sans compteur du tout. Et comme Python, `Rc` seul ne voit pas les cycles (10.5) ; Python a un ramasse-miettes de secours pour ça, Rust n'en a pas.

### 10.4 `RefCell<T>` et `Cell<T>` : la mutabilité intérieure

La règle du chapitre 4, « un `&mut` exclusif ou des `&` partagés », est vérifiée à la compilation. Certaines structures la respectent mais d'une façon que le compilateur ne peut pas prouver : plusieurs `Rc` vers un cache qu'on remplit, un compteur incrémenté dans une méthode qui ne prend que `&self`. `RefCell<T>` déplace la vérification **à l'exécution** : `borrow()` rend un emprunt partagé, `borrow_mut()` un emprunt exclusif, et RefCell compte les emprunts en cours. Si la règle est violée, le programme panique au lieu de ne pas compiler. `Cell<T>` est la version sans emprunt, pour les petits types `Copy` : `get()` copie la valeur dehors, `set()` en écrit une nouvelle, et aucune référence ne sort jamais, donc rien à vérifier.

```rust
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Un cache qu'on remplit à travers &self : la mutabilité intérieure.
struct Annuaire {
    entrees: RefCell<Vec<String>>,
    lectures: Cell<u32>,
}

impl Annuaire {
    fn ajouter(&self, nom: &str) {
        self.entrees.borrow_mut().push(nom.to_string());
    }

    fn nombre(&self) -> usize {
        self.lectures.set(self.lectures.get() + 1);
        self.entrees.borrow().len()
    }
}

fn main() {
    let annuaire = Rc::new(Annuaire {
        entrees: RefCell::new(Vec::new()),
        lectures: Cell::new(0),
    });
    let copie = Rc::clone(&annuaire);
    annuaire.ajouter("db1");
    copie.ajouter("db2");
    println!("{} entrées, vues par l'autre Rc : {}", annuaire.nombre(), copie.nombre());
    println!("lectures : {}", annuaire.lectures.get());

    let lecture = annuaire.entrees.borrow();
    match annuaire.entrees.try_borrow_mut() {
        Ok(_) => println!("emprunt mutable accordé"),
        Err(e) => println!("try_borrow_mut pendant une lecture : {e}"),
    }
    println!("première entrée : {}", lecture[0]);
    drop(lecture);
    annuaire.entrees.borrow_mut().clear();

    let lecture = annuaire.entrees.borrow();
    annuaire.ajouter("db3"); // panique : une lecture est en cours
    println!("{}", lecture.len());
}
```

```
$ cargo run -q
2 entrées, vues par l'autre Rc : 2
lectures : 2
try_borrow_mut pendant une lecture : RefCell already borrowed
première entrée : db1

thread 'main' (55366596) panicked at src/bin/c10_refcell.rs:12:22:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo $?
101
```

`Rc<RefCell<T>>` est le motif complet : plusieurs propriétaires (Rc) qui peuvent modifier (RefCell). Les emprunts rendus par `borrow()` et `borrow_mut()` sont des valeurs (`Ref` et `RefMut`) qui relâchent l'emprunt dans leur `Drop` : un `let lecture = x.borrow();` garde l'emprunt jusqu'à la fin du bloc, alors que `x.borrow().len()` le relâche à la fin de l'instruction. C'est presque toujours la source d'une panique `already borrowed` : un emprunt rangé dans une variable qui vit plus longtemps que prévu. `try_borrow_mut()` rend un `Result` au lieu de paniquer, pour les cas où le conflit est possible et prévu. La panique pointe sur la ligne 12, dans `ajouter` : c'est là que `borrow_mut()` a été appelé, et l'emprunt fautif est ailleurs, à toi de le trouver.

**Piège :** les versions plus anciennes de Rust disaient `already borrowed: BorrowMutError`, c'est la même panique. Et `RefCell` n'est pas un moyen de faire taire le vérificateur d'emprunts : c'est un vérificateur à l'exécution, avec un coût (un compteur par emprunt) et un risque (une panique en production au lieu d'une erreur de compilation). Garde-le pour les graphes, les caches et les observateurs, où le partage est vraiment dynamique.

### 10.5 `Weak` et les cycles

Un compteur de références ne libère jamais un cycle : si A tient un `Rc` vers B et B un `Rc` vers A, chacun garde l'autre à au moins 1, même quand plus personne ne les voit. Le cas d'école est l'arbre où chaque nœud connaît son parent. La règle : **`Rc` dans le sens de la possession, `Weak` dans l'autre**. Un `Weak` (obtenu par `Rc::downgrade`) ne compte pas comme propriétaire ; `upgrade()` rend `Some(Rc)` si la valeur existe encore, `None` sinon.

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Dossier {
    nom: String,
    parent: RefCell<Weak<Dossier>>,
    enfants: RefCell<Vec<Rc<Dossier>>>,
}

impl Drop for Dossier {
    fn drop(&mut self) {
        println!("drop {}", self.nom);
    }
}

fn dossier(nom: &str) -> Rc<Dossier> {
    Rc::new(Dossier {
        nom: nom.to_string(),
        parent: RefCell::new(Weak::new()),
        enfants: RefCell::new(Vec::new()),
    })
}

// La version qui fuit : deux Rc qui se tiennent l'un l'autre.
struct Noeud {
    nom: String,
    autre: RefCell<Option<Rc<Noeud>>>,
}

impl Drop for Noeud {
    fn drop(&mut self) {
        println!("drop {}", self.nom);
    }
}

fn main() {
    {
        let etc = dossier("etc");
        let nginx = dossier("nginx");
        *nginx.parent.borrow_mut() = Rc::downgrade(&etc);
        etc.enfants.borrow_mut().push(Rc::clone(&nginx));

        let parent = nginx.parent.borrow().upgrade();
        println!("parent de nginx : {:?}", parent.map(|p| p.nom.clone()));
        println!("etc : {} fort, {} faible", Rc::strong_count(&etc), Rc::weak_count(&etc));
        println!("nginx : {} forts", Rc::strong_count(&nginx));
    }
    println!("-- fin du bloc avec Weak");

    {
        let a = Rc::new(Noeud { nom: "a".into(), autre: RefCell::new(None) });
        let b = Rc::new(Noeud { nom: "b".into(), autre: RefCell::new(Some(Rc::clone(&a))) });
        *a.autre.borrow_mut() = Some(Rc::clone(&b));
        println!("a : {} forts, b : {} forts", Rc::strong_count(&a), Rc::strong_count(&b));
    }
    println!("-- fin du bloc avec deux Rc : aucun drop, la mémoire a fui");
}
```

```
$ cargo run -q
parent de nginx : Some("etc")
etc : 1 fort, 1 faible
nginx : 2 forts
drop etc
drop nginx
-- fin du bloc avec Weak
a : 2 forts, b : 2 forts
-- fin du bloc avec deux Rc : aucun drop, la mémoire a fui
```

Avec le `Weak`, `etc` n'a qu'un propriétaire (sa variable) : en sortant du bloc, il est détruit, ce qui détruit son `Vec` d'enfants, ce qui fait tomber `nginx` à zéro. Avec deux `Rc`, rien n'est jamais détruit. Note que c'est une **fuite**, pas un bug de sécurité : Rust garantit l'absence d'accès à de la mémoire libérée, pas l'absence de fuite (`std::mem::forget` et `Box::leak` sont d'ailleurs des fonctions sûres). Le labo 10 le prouve en comptant les `Drop` avec un compteur atomique global.

### 10.6 `Drop` et l'ordre de destruction

`Drop` est le destructeur : la méthode `drop(&mut self)` appelée automatiquement quand une valeur cesse d'exister. C'est elle qui ferme un fichier, libère un tampon, rend une connexion à un pool, relâche un verrou. L'ordre est fixé, et il vaut la peine de le voir une fois :

```rust
struct Bruyant(&'static str);

impl Drop for Bruyant {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

#[allow(dead_code)]
struct Paire {
    premier: Bruyant,
    second: Bruyant,
}

fn main() {
    let _a = Bruyant("a");
    let _b = Bruyant("b");
    let _p = Paire { premier: Bruyant("p.premier"), second: Bruyant("p.second") };
    let v = vec![Bruyant("v[0]"), Bruyant("v[1]")];
    let _ = Bruyant("ignoré"); // `_` seul : détruit tout de suite
    drop(v);
    println!("fin de main");
}
```

```
$ cargo run -q
drop ignoré
drop v[0]
drop v[1]
fin de main
drop p.premier
drop p.second
drop b
drop a
```

Les variables sont détruites **dans l'ordre inverse de leur déclaration** (`_p`, puis `_b`, puis `_a`) : ce qui a été créé après peut dépendre de ce qui a été créé avant, l'inverse non. Les champs d'une struct et les éléments d'un `Vec` le sont **dans l'ordre**. `drop(v)` détruit tout de suite (ce n'est qu'une fonction qui prend la valeur et ne fait rien : la fin de sa portée fait le travail). Et `let _ = ...` ne lie rien : la valeur est détruite à la fin de l'instruction, alors que `let _a = ...` la garde jusqu'à la fin du bloc. **Piège** classique : `let _ = mutex.lock();` relâche le verrou aussitôt.

Le `Drop` par défaut d'une structure récursive est récursif : une liste chaînée d'un million de `Box` se libère par un million d'appels imbriqués, et le thread meurt avec `thread 'main' has overflowed its stack`. Le labo 10 te le fait provoquer, puis corriger par un `Drop` écrit à la main qui défait la chaîne dans une boucle.

### 10.7 Durées de vie avancées

Une struct peut contenir une référence. Elle doit alors dire **combien de temps** la donnée empruntée doit vivre, par un paramètre de durée de vie :

```rust
// Une vue sur une ligne de journal : elle EMPRUNTE le texte, sans le copier.
struct Ligne<'a> {
    niveau: &'a str,
    message: &'a str,
}

impl<'a> Ligne<'a> {
    fn lire(texte: &'a str) -> Option<Ligne<'a>> {
        let (niveau, message) = texte.split_once(' ')?;
        Some(Ligne { niveau, message })
    }

    // Le résultat vit aussi longtemps que le texte, pas que la Ligne.
    fn message(&self) -> &'a str {
        self.message
    }
}

const PREFIXE: &str = "app"; // &'static str : gravé dans le binaire

fn main() {
    let journal = String::from("ERROR disque plein\nINFO tout va bien");
    let lignes: Vec<Ligne> = journal.lines().filter_map(Ligne::lire).collect();
    for l in &lignes {
        println!("{PREFIXE} [{}] {}", l.niveau, l.message());
    }
}
```

```
$ cargo run -q
app [ERROR] disque plein
app [INFO] tout va bien
```

`Ligne<'a>` se lit « une ligne qui emprunte quelque chose pour la durée `'a` ». `impl<'a> Ligne<'a>` déclare le paramètre pour le bloc, comme `impl<T> Liste<T>` pour un type. La signature de `message` dit quelque chose de précis : le `&str` rendu est lié à `'a`, le texte d'origine, et pas à `&self`, la `Ligne`. On peut donc jeter la `Ligne` et garder le message, tant que le texte vit. L'inverse ne compile pas :

```rust
    let message;
    {
        let texte = String::from("WARN lent");
        let l = Ligne::lire(&texte).unwrap();
        message = l.message();
    }
    println!("{message}");
```

```
$ cargo build
error[E0597]: `texte` does not live long enough
  --> src/bin/c10_vie.rs:31:29
   |
30 |         let texte = String::from("WARN lent");
   |             ----- binding `texte` declared here
31 |         let l = Ligne::lire(&texte).unwrap();
   |                             ^^^^^^ borrowed value does not live long enough
32 |         message = l.message();
33 |     }
   |     - `texte` dropped here while still borrowed
34 |     println!("{message}");
   |                ------- borrow later used here
```

Trois lieux, trois marques : où la valeur est déclarée, où elle meurt, où l'emprunt est encore utilisé. C'est la forme de toutes les erreurs de durée de vie ; lis-les dans cet ordre.

`'static` est la durée de vie qui dure tout le programme : les littéraux (`"app"` est un `&'static str`, ses octets sont dans le binaire), les `static`, et ce qu'on a volontairement fait fuir avec `Box::leak`. Attention à la lecture de `T: 'static` dans une contrainte (tu la verras sur `thread::spawn` au chapitre 11) : elle ne veut pas dire « vit pour toujours » mais « ne contient aucune référence de durée limitée ». Une `String` est `'static` au sens de la contrainte, puisqu'elle possède tout.

**La variance, en deux phrases.** `&'a T` est covariant : une `&'static str` s'utilise partout où une `&'a str` plus courte est attendue, le compilateur raccourcit tout seul. `&mut T` est invariant en `T` : on ne peut pas faire passer un `&mut &'static str` pour un `&mut &'a str`, sinon on pourrait y écrire une référence courte que l'autre côté croirait éternelle.

**Quand arrêter les durées de vie.** Une struct avec une référence est la bonne solution pour une **vue temporaire** : un analyseur qui découpe un texte sans le copier, un itérateur. Elle devient la mauvaise dès que la struct doit être stockée, renvoyée de loin ou envoyée à un thread : les `'a` contaminent tout ce qui la contient. Possède alors la donnée : `String` au lieu de `&str`, `Box<T>` ou `Rc<T>` (`Arc<T>` entre threads). Copier cinquante octets coûte moins cher qu'une après-midi contre le vérificateur d'emprunts.

### 10.8 Les abstractions à coût nul

La promesse de Rust, héritée du C++ : ce que tu n'utilises pas ne coûte rien, et ce que tu utilises, tu ne pourrais pas l'écrire mieux à la main. Vérifions-la sur une somme des carrés des nombres pairs, écrite deux fois dans une bibliothèque :

```rust
// La somme des carrés des nombres pairs, écrite deux fois.

#[unsafe(no_mangle)]
pub fn somme_iterateurs(v: &[u64]) -> u64 {
    v.iter().filter(|x| *x % 2 == 0).map(|x| x * x).sum()
}

#[unsafe(no_mangle)]
pub fn somme_boucle(v: &[u64]) -> u64 {
    let mut total = 0;
    let mut i = 0;
    while i < v.len() {
        let x = v[i];
        if x % 2 == 0 {
            total += x * x;
        }
        i += 1;
    }
    total
}
```

`#[unsafe(no_mangle)]` garde le nom tel quel dans le binaire, pour le retrouver dans l'assembleur (l'attribut est marqué `unsafe` depuis l'édition 2024, parce que deux symboles de même nom feraient un comportement indéfini à l'édition de liens). Demandons l'assembleur à `rustc` :

```
$ cargo rustc -q --release --lib -- --emit asm
$ awk '/^_somme_iterateurs:/,/\.cfi_endproc/' target/release/deps/zc-*.s | wc -l
      61
$ awk '/^_somme_boucle:/,/\.cfi_endproc/' target/release/deps/zc-*.s | wc -l
      61
```

Soixante et une lignes chacune. Le cœur des deux fonctions est la même boucle, instruction pour instruction ; seuls l'ordre des premières instructions et les noms d'étiquettes diffèrent. La voici, côté itérateurs :

```
LBB1_5:
	ldp	x15, x16, [x13, #-16]
	ldp	x17, x2, [x13], #32
	mul	x3, x15, x15
	mul	x4, x16, x16
	mul	x5, x17, x17
	mul	x6, x2, x2
	tst	x15, #0x1
	csel	x15, x3, xzr, eq
	...
	subs	x14, x14, #4
	b.ne	LBB1_5
```

LLVM a déroulé la boucle par quatre (deux `ldp` chargent quatre `u64`), remplacé le `if` par des `csel` (un choix sans branchement) et supprimé toute vérification de bornes. Les fermetures, `filter`, `map` et `sum` ont disparu, entièrement inlinées. Tu peux donc écrire des chaînes d'itérateurs (chapitre 5) dans une boucle chaude sans t'excuser : elles ne sont pas plus lentes qu'un `for` indexé, et souvent plus rapides, parce que l'itérateur prouve au compilateur qu'on ne sort pas des bornes.

**Venant de Go :** c'est la différence de philosophie la plus nette entre les deux langages. Le compilateur Go optimise peu, pour compiler vite, et une chaîne de fonctions d'ordre supérieur y coûte de vrais appels. Le compilateur Rust optimise énormément, et compile lentement en échange (10.10).

### 10.9 Monomorphisation contre `dyn`

Une fonction générique `fn f<T: Display>(x: T)` n'existe pas dans le binaire telle quelle : le compilateur en fabrique **une copie par type concret utilisé**, chacune optimisée pour son type. C'est la monomorphisation. Une fonction `fn f(x: &dyn Display)` existe en un seul exemplaire et appelle les méthodes par la vtable, à l'exécution. Comptons :

```rust
use std::fmt::Display;

#[inline(never)]
fn generique<T: Display>(x: T) -> usize {
    format!("[{x}]").len()
}

#[inline(never)]
fn dynamique(x: &dyn Display) -> usize {
    format!("[{x}]").len()
}

fn main() {
    let n = generique(42) + generique(1.5) + generique("db1") + generique('x');
    let m = dynamique(&42) + dynamique(&1.5) + dynamique(&"db1") + dynamique(&'x');
    println!("{n} {m}");
}
```

```
$ cargo run -q --release
17 17
$ nm -C target/release/c10_mono | grep -E 'generique|dynamique'
00000001000009cc t c10_mono::dynamique::h8cee9634f9bb1cb4
0000000100000a30 t c10_mono::generique::h904a7310fea8aba0
0000000100000a98 t c10_mono::generique::ha9babf4ffa0f714c
0000000100000b08 t c10_mono::generique::hab46572926c2835e
0000000100000b70 t c10_mono::generique::hacd95b061f77bba6
```

Quatre `generique`, un `dynamique`. Le générique est plus rapide (appel direct, inlinable, optimisé pour le type), le `dyn` est plus compact (un seul corps) et compile plus vite. Le choix par défaut est le générique ; prends `dyn` quand tu as besoin de mélanger des types dans une collection (10.2), quand le type n'est connu qu'à l'exécution (un plugin, une sonde choisie par la configuration), ou quand une bibliothèque très générique fait exploser la taille du binaire et le temps de compilation. C'est une des raisons pour lesquelles un binaire Rust qui utilise beaucoup de génériques pèse plusieurs méga-octets en debug (le `dsk` du labo 12 fait 4,2 Mo en debug, 1,1 Mo en release).

### 10.10 Ce que fait le compilateur, et pourquoi c'est lent

`rustc` travaille en étages. Le code est analysé, les macros développées, les noms résolus, puis abaissé en **HIR**, où ont lieu le typage et la résolution des traits, puis en **MIR** (*mid-level IR*), un graphe de blocs élémentaires sur lequel tourne le **vérificateur d'emprunts** : c'est sur le MIR que « `texte` does not live long enough » est calculé. Le MIR monomorphisé est enfin traduit pour **LLVM**, le moteur de Clang, qui optimise et produit le code machine.

Pourquoi c'est lent, sur le labo 12 (une CLI avec clap, serde, serde_json, anyhow, 28 crates dans l'arbre), sur un Mac M1 :

```
$ cargo build                  # tout depuis zéro
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.13s
$ touch src/main.rs && cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.77s
$ cargo build --release        # après le build de debug
    Finished `release` profile [optimized] target(s) in 9.01s
$ touch src/main.rs && cargo build --release
    Finished `release` profile [optimized] target(s) in 0.96s
```

Trois raisons. **La crate est l'unité de compilation** : tout `serde_json` est compilé d'un bloc, et les macros `derive` sont des programmes qu'il faut compiler avant de les exécuter. **La monomorphisation** fait générer une copie de chaque fonction générique par type, et c'est LLVM qui prend l'essentiel du temps. **L'optimisation** est poussée. En échange, la compilation **incrémentale** (active en debug) ne refait que ce qui dépend de ce qui a changé, et les dépendances ne sont jamais recompilées tant que `Cargo.lock` ne bouge pas. Quand ça devient pénible : `cargo check`, qui s'arrête avant LLVM (typage et emprunts seulement, c'est ce que ton éditeur lance), et le découpage en crates d'un espace de travail (chapitre 9).

**Venant de Go :** depuis zéro, Go compile un outil de cette taille plusieurs fois plus vite. C'est le prix conscient de 10.8 et de 10.9, et la principale raison de garder Go pour les outils qu'on modifie vingt fois par jour (chapitre 0).

### 10.11 `unsafe` : ce qu'il débloque, ce qu'il ne débloque pas

Le compilateur ne peut pas tout prouver. Un allocateur, une liste doublement chaînée efficace, un appel à une bibliothèque C, l'accès au matériel : ce code est correct pour des raisons que le système de types ne sait pas exprimer. `unsafe` est la porte : **un bloc `unsafe { }` autorise cinq opérations de plus**, et c'est tout.

1. Déréférencer un pointeur brut (`*const T`, `*mut T`).
2. Appeler une fonction `unsafe`, dont les fonctions C déclarées par FFI (10.12).
3. Lire ou écrire une variable `static mut`.
4. Implémenter un trait `unsafe` (`Send` et `Sync` à la main, chapitre 11).
5. Accéder aux champs d'une `union`.

```rust
fn main() {
    let mut x = 10;
    let p = &raw mut x; // un pointeur brut : *mut i32
    let q = &raw const x;
    // Créer un pointeur brut est sûr ; le déréférencer ne l'est pas.
    unsafe {
        *p += 1;
        println!("x = {}, via q : {}", *p, *q);
    }
}
```

```
$ cargo run -q
x = 11, via q : 11
```

Ce qu'`unsafe` **ne débloque pas** : le vérificateur d'emprunts, le typage, les vérifications de bornes de `v[i]`. Le code à l'intérieur reste du Rust :

```rust
    let v = vec![1, 2, 3];
    let r1 = &mut x;
    unsafe {
        let r2 = &mut x;
        *r2 += *r1 + v[0];
    }
```

```
$ cargo build
warning: unnecessary `unsafe` block
error[E0499]: cannot borrow `x` as mutable more than once at a time
  --> src/bin/c10_unsafe.rs:14:18
   |
12 |     let r1 = &mut x;
   |              ------ first mutable borrow occurs here
13 |     unsafe {
14 |         let r2 = &mut x;
   |                  ^^^^^^ second mutable borrow occurs here
```

Le compilateur te dit même que le bloc est inutile : rien dedans n'en a besoin. Le mot ne veut pas dire « ici, pas de règles », il veut dire « ici, **j'ai vérifié moi-même** des règles que le compilateur ne voit pas ». Si tu te trompes, le résultat est un **comportement indéfini** (*undefined behavior*, UB) : le programme n'a plus de sens, et l'optimiseur a le droit d'en faire n'importe quoi. Ce n'est pas une erreur qui se rattrape :

```rust
fn main() {
    let v = vec![1u32, 2, 3];
    // get_unchecked saute la vérification des bornes : c'est à NOUS de
    // garantir que l'indice est valide. Ici il ne l'est pas.
    let x = unsafe { *v.get_unchecked(1000) };
    println!("v[1000] = {x}");
}
```

```
$ cargo run -q ; echo "code $?"
thread 'main' (55376625) panicked at src/bin/c10_ub.rs:5:25:
unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice

This indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.
thread caused non-unwinding panic. aborting.
code 134
$ cargo run -q --release ; echo "code $?"
code 133
```

En debug, la bibliothèque standard vérifie la précondition et s'arrête avec un message, en prévenant qu'elle n'est pas obligée de le faire. En release, la vérification disparaît, LLVM sait que l'indice 1000 « ne peut pas arriver », et le programme s'arrête sur une instruction piège sans rien afficher, pas même la ligne `println!`. Une autre version du compilateur pourrait afficher un nombre au hasard, ou continuer avec une mémoire corrompue. C'est ce qu'UB veut dire.

Les règles du bon usage : un bloc `unsafe` minuscule, précédé d'un commentaire `// SAFETY :` qui dit **pourquoi** chaque précondition est remplie (clippy peut l'exiger avec `undocumented_unsafe_blocks`), et enfermé dans une **fonction sûre** dont la signature rend la mauvaise utilisation impossible. C'est ainsi qu'est écrite la bibliothèque standard : `Vec` contient de l'`unsafe`, et tu n'en vois rien. Pour tester ce code, **Miri** (`rustup +nightly component add miri`, puis `cargo +nightly miri test`) exécute les tests dans un interpréteur qui vérifie chaque accès mémoire et dénonce un UB que le lancement normal laisserait passer : le détecteur de courses de Go, pour la mémoire. C'est la seule entorse de ce cours à la règle « stable seulement », et pour un outil de vérification.

Les `static mut` méritent un mot : l'édition 2024 refuse d'en prendre une référence.

```
$ cargo build
error: creating a shared reference to mutable static
 --> src/bin/c10_static.rs:9:17
  |
9 |         let r = &REQUETES;
  |                 ^^^^^^^^^ shared reference to mutable static
  |
  = note: shared references to mutable statics are dangerous; it's undefined behavior if the static is mutated or if a mutable reference is created for it while the shared reference lives
  = note: `#[deny(static_mut_refs)]` (part of `#[deny(rust_2024_compatibility)]`) on by default
```

N'en écris pas. Un compteur global est un `static REQUETES: AtomicU64 = AtomicU64::new(0);` (chapitre 11), sûr et sans verrou ; une donnée globale initialisée une fois est un `static CONFIG: OnceLock<Config>`. Le labo 10 compte ses `Drop` avec un `AtomicUsize` statique.

### 10.12 FFI : appeler du C, `unsafe extern "C"` et `#[repr(C)]`

Rust parle le C sans glue : tu déclares la signature de la fonction C, l'éditeur de liens la trouve dans la libc, déjà liée à tout programme. Pas de cgo, pas de coût d'appel : un appel de fonction C depuis Rust est un appel de fonction, comme en C.

```rust
use std::ffi::{CStr, CString, c_char, c_int};

// Les signatures C, recopiées des pages de manuel. Le mot `unsafe` du bloc
// dit que c'est NOUS qui garantissons qu'elles sont justes.
unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize; // size_t strlen(const char *s);
    safe fn getpid() -> c_int; // pid_t getpid(void); aucune précondition
    fn getenv(nom: *const c_char) -> *const c_char; // char *getenv(const char *name);
}

/// Une enveloppe sûre : &CStr garantit un pointeur valide terminé par NUL.
fn longueur(s: &CStr) -> usize {
    // SAFETY : s.as_ptr() est valide et terminé par NUL pendant l'appel.
    unsafe { strlen(s.as_ptr()) }
}

fn variable(nom: &str) -> Option<String> {
    let nom = CString::new(nom).ok()?; // refuse un nom qui contient un NUL
    // SAFETY : nom est une chaîne C valide ; getenv rend NULL ou un pointeur
    // vers une chaîne de l'environnement, qu'on copie aussitôt.
    unsafe {
        let p = getenv(nom.as_ptr());
        if p.is_null() {
            None
        } else {
            Some(CStr::from_ptr(p).to_string_lossy().into_owned())
        }
    }
}

fn main() {
    println!("strlen = {}", longueur(c"sous le capot"));
    println!("getpid = {} (std::process::id = {})", getpid(), std::process::id());
    println!("HOME   = {:?}", variable("HOME"));
    println!("ABSENT = {:?}", variable("CETTE_VARIABLE_N_EXISTE_PAS"));
}
```

```
$ cargo run -q
strlen = 13
getpid = 22216 (std::process::id = 22216)
HOME   = Some("/Users/stranix")
ABSENT = None
```

Depuis l'édition 2024, le bloc s'écrit `unsafe extern "C"` : c'est la déclaration elle-même qui est une promesse non vérifiable (un `usize` à la place d'un `int`, et l'appel rend des valeurs fausses, ou pire, sans que rien ne le signale). Chaque fonction y est `unsafe` à appeler par défaut, sauf si on la marque `safe fn`, comme `getpid` qui n'a aucune précondition. Appeler `strlen` hors d'un bloc `unsafe` :

```
$ cargo build
error[E0133]: call to unsafe function `strlen` is unsafe and requires unsafe block
 --> src/bin/c10_e133.rs:8:20
  |
8 |     println!("{}", strlen(c"abc".as_ptr()));
  |                    ^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
```

Les types du pont sont dans `std::ffi` : `c_char`, `c_int`, `c_long` (de taille variable selon la plateforme, comme en C), `CStr` pour une chaîne C empruntée, `CString` pour en fabriquer une, et le littéral `c"..."` (Rust 1.77) pour une `&CStr` constante. Une struct passée à du C doit être `#[repr(C)]`, sinon Rust réordonne ses champs (10.1). Pour une vraie bibliothèque, on ne recopie pas les signatures : `bindgen` les génère depuis le `.h`, et la crate `libc` les fournit pour toute la libc. Le labo 10 s'en passe exprès, pour que tu écrives le bloc toi-même.

**Venant du C :** tes bibliothèques C s'utilisent telles quelles, et l'inverse aussi : une `#[unsafe(no_mangle)] pub extern "C" fn` s'appelle depuis du C, et `crate-type = ["cdylib"]` produit un `.so`. C'est comme ça qu'on réécrit un programme C module par module, et ce qui permet au noyau Linux d'accueillir des pilotes Rust. Les pointeurs bruts sont tes pointeurs C, arithmétique et `NULL` compris ; les références sont des pointeurs que le compilateur prouve valides, alignés et non nuls. Une enveloppe FFI passe des uns aux autres en vérifiant ce que le C ne garantit pas.

**Venant de Go :** cgo coûte cher (des dizaines de nanosecondes par appel, un changement de pile, l'ordonnanceur prévenu) et casse la compilation croisée. En Rust, un appel C coûte un appel, et la compilation croisée marche tant qu'on a la bibliothèque C de la cible. Les goroutines, qui sont la vraie raison d'être de l'exécutif Go, n'ont pas d'équivalent dans la bibliothèque standard Rust : des threads système au chapitre 11, des tâches async au chapitre 13.

### 10.13 Pour le labo

Le [labo 10](../labs/10-sous-le-capot/README.md) reprend chaque mécanisme dans une petite expérience, un module par expérience, comme le labo Go du même nom : un compteur partagé en `Rc<RefCell<T>>` dont les tests vérifient `Rc::strong_count` et une `BorrowMutError` attrapée par `try_borrow_mut`, une liste chaînée en `Option<Box<Noeud<T>>>` avec son itérateur et un `Drop` itératif qui libère un million de maillons sans exploser la pile, un arbre avec des `Weak` vers le parent dont un test prouve l'absence de fuite en comptant les `Drop`, `strlen` et `getpid` appelés par FFI derrière des enveloppes sûres, et la taille d'une vingtaine de types, niche comprise.

### À retenir

- Toute valeur vit sur la pile ou dans une autre valeur ; le tas n'apparaît que par un type qui le demande (`Box`, `Vec`, `String`, `Rc`), lui-même petit et de taille fixe.
- `size_of` et `align_of` disent la vérité ; Rust réordonne les champs, `#[repr(C)]` l'interdit ; `Option<Box<T>>` ne coûte rien de plus grâce à la niche.
- `Box<T>` : un propriétaire, sur le tas ; pour les types récursifs, les `dyn Trait` et les très grosses valeurs, rien d'autre.
- `Rc<T>` : plusieurs propriétaires dans un thread, `Rc::clone` incrémente un compteur ; `RefCell<T>` déplace la règle des emprunts à l'exécution et panique (`RefCell already borrowed`) si on la viole ; `Cell<T>` pour un petit `Copy`.
- `Rc` dans le sens de la possession, `Weak` dans l'autre, sinon le cycle fuit ; une fuite n'est pas un UB, Rust ne la promet pas impossible.
- Variables détruites dans l'ordre inverse de leur déclaration, champs dans l'ordre ; `let _ = x` détruit aussitôt ; un `Drop` récursif sur une longue chaîne fait déborder la pile.
- Une struct avec une référence (`Ligne<'a>`) pour une vue temporaire ; dès qu'elle doit être stockée ou envoyée, possède la donnée.
- Itérateurs et fermetures compilent en la même boucle qu'à la main ; les génériques sont copiés par type (rapides, gros), `dyn` passe par une vtable (compact, un appel indirect).
- `rustc` vérifie les emprunts sur le MIR et laisse LLVM optimiser : compilation lente, `cargo check` pour itérer, incrémental pour ta crate.
- `unsafe` débloque cinq opérations et aucune règle ; un bloc minuscule, un `// SAFETY :`, une enveloppe sûre, Miri pour vérifier ; FFI par `unsafe extern "C"`, `safe fn` quand il n'y a pas de précondition.

---

← [9. Modules, crates, Cargo, tests, documentation](09-modules-crates-tests.md) · [Sommaire](../README.md) · [11. Concurrence : threads, channels, Arc et Mutex, Send et Sync](11-concurrence.md) →
