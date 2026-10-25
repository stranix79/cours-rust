# 6. Structs, enums, méthodes, Option

*Rust de zéro à la prod : chapitre 6 sur 16.* ← [5. Collections et itérateurs](05-collections-et-iterateurs.md) · [Sommaire](../README.md) · [7. Traits et génériques](07-traits-et-generiques.md) →

Jusqu'ici, tes données étaient des entiers, des chaînes et des collections de ceux-ci. Ce chapitre te donne les deux briques avec lesquelles on modélise tout le reste : la **struct**, qui regroupe des champs (un hôte a un nom, un port, un état), et l'**enum**, qui dit qu'une valeur est *l'une ou l'autre* de plusieurs formes (un événement est un démarrage, ou une sonde avec sa latence, ou une panne avec le nom de l'hôte). On y attache des méthodes avec `impl`, et le chapitre 4 décide de leur signature.

La struct, tu la connais du C et de Go. L'enum avec données, non : c'est la pièce que Go n'a pas, et c'est elle qui permet à Rust de se passer de `null`. `Option<T>`, que tu croises depuis le chapitre 5 sans trop savoir ce que c'est, n'est qu'un enum de la bibliothèque standard à deux variantes. Quand tu l'auras compris, tu sauras pourquoi un `NullPointerException` ne peut pas arriver en Rust.

### 6.1 Trois sortes de structs

```rust
#[derive(Debug)]
struct Hote {
    nom: String,
    port: u16,
    actif: bool,
}

#[derive(Debug)]
struct Celsius(f64);

#[derive(Debug)]
struct Point(i32, i32);

#[derive(Debug)]
struct Invite;

fn main() {
    let nom = String::from("db-01");
    let mut h = Hote {
        nom,
        port: 5432,
        actif: true,
    };
    h.port = 6432;
    println!("{} écoute sur {} (actif : {})", h.nom, h.port, h.actif);
    println!("{h:?}");
    println!("{h:#?}");

    let t = Celsius(21.5);
    let p = Point(3, -1);
    println!("{} °C, x={} y={}", t.0, p.0, p.1);
    println!("{:?} {:?} {:?}", t, p, Invite);
}
```

```
$ cargo run
db-01 écoute sur 6432 (actif : true)
Hote { nom: "db-01", port: 6432, actif: true }
Hote {
    nom: "db-01",
    port: 6432,
    actif: true,
}
21.5 °C, x=3 y=-1
Celsius(21.5) Point(3, -1) Invite
```

- La **struct à champs nommés** est la plus courante. On la construit en donnant chaque champ, dans n'importe quel ordre, et **tous** les champs : il n'y a pas de valeur zéro implicite. Quand une variable porte le nom du champ, `nom` suffit au lieu de `nom: nom`.
- La mutabilité est celle de la variable, pas des champs : `let mut h` rend tous les champs modifiables, `let h` aucun. Pas de champ `const` au milieu d'une struct mutable.
- La **struct tuple** (`Celsius(f64)`) a des champs sans nom, qu'on lit avec `.0`, `.1`. Elle sert surtout à donner un type distinct à une valeur simple : un `Celsius` et un `Fahrenheit` sont deux `f64`, mais le compilateur refuse de passer l'un pour l'autre. C'est le motif *newtype*, qu'on retrouve au chapitre 7.
- La **struct unité** (`Invite`) n'a aucun champ et ne prend aucune place en mémoire. Elle sert à porter un trait (chapitre 7) ou un marqueur.
- `#[derive(Debug)]` demande au compilateur d'écrire le code qui affiche la struct avec `{:?}`, et `{:#?}` l'affiche sur plusieurs lignes, indentée. Sans lui, `println!("{h:?}")` ne compile pas.

**Venant du C :** même disposition en mémoire, à un détail près : sans `#[repr(C)]`, le compilateur a le droit de **réordonner les champs** pour réduire le remplissage. Tu ne peux donc pas supposer l'ordre des octets d'une struct Rust, sauf à le demander (et tu ne le demandes que pour parler à du C, chapitre 10).

**Venant de Go :** c'est la struct Go, sans valeur zéro et sans champs exportés par majuscule (section 6.8). `Hote{}` en Go donne une struct remplie de zéros ; en Rust, `Hote {}` ne compile pas (`missing fields`), et la valeur par défaut, si tu en veux une, se demande explicitement (section 6.3).

### 6.2 `impl` : méthodes et fonctions associées

Les méthodes d'un type se déclarent dans un bloc `impl`, séparé de la définition. Leur premier paramètre dit **comment elles accèdent à la valeur**, et ce sont exactement les trois cas du chapitre 4 :

```rust
#[derive(Debug)]
struct Pool {
    hote: String,
    max: usize,
    ouvertes: usize,
}

impl Pool {
    fn new(hote: &str, max: usize) -> Self {
        Self {
            hote: hote.to_string(),
            max,
            ouvertes: 0,
        }
    }

    fn libres(&self) -> usize {
        self.max - self.ouvertes
    }

    fn ouvrir(&mut self) -> bool {
        if self.ouvertes == self.max {
            return false;
        }
        self.ouvertes += 1;
        true
    }

    fn fermer(self) -> String {
        format!("pool {} fermé avec {} connexions", self.hote, self.ouvertes)
    }
}

fn main() {
    let mut pool = Pool::new("db-01", 2);
    println!("libres : {}", pool.libres());
    println!("{} {} {}", pool.ouvrir(), pool.ouvrir(), pool.ouvrir());
    println!("libres : {}", pool.libres());
    let bilan = pool.fermer();
    println!("{bilan}");
}
```

```
$ cargo run
libres : 2
true true false
libres : 0
pool db-01 fermé avec 2 connexions
```

- `fn new(...) -> Self` n'a pas de `self` : c'est une **fonction associée**, appelée avec `Pool::new(...)`. `new` n'est pas un mot-clé, juste la convention pour le constructeur principal. `Self` désigne le type du bloc `impl`, ici `Pool`.
- `fn libres(&self)` **emprunte** la valeur en lecture. C'est le cas par défaut : une méthode qui calcule, affiche, compare.
- `fn ouvrir(&mut self)` l'emprunte **en écriture**. L'appelant doit avoir une variable `mut`, et personne d'autre ne regarde le pool pendant l'appel.
- `fn fermer(self)` **prend possession** de la valeur. Après l'appel, le pool n'existe plus pour l'appelant : il a été déplacé dans la méthode, qui en fait ce qu'elle veut (ici, elle le détruit après avoir écrit un bilan).

`pool.ouvrir()` est un raccourci : Rust ajoute le `&mut` tout seul (`Pool::ouvrir(&mut pool)`). C'est pour ça que la méthode s'appelle de la même façon, quel que soit son receveur. Mais le compilateur vérifie. Retire le `mut` de `let mut pool` :

```
$ cargo build
error[E0596]: cannot borrow `pool` as mutable, as it is not declared as mutable
  --> src/main.rs:35:9
   |
35 |     let pool = Pool::new("db-01", 2);
   |         ^^^^ not mutable
36 |     println!("libres : {}", pool.libres());
37 |     println!("{} {} {}", pool.ouvrir(), pool.ouvrir(), pool.ouvrir());
   |                          ----           ----           ---- cannot borrow as mutable
```

Et utilise le pool après l'avoir fermé :

```
$ cargo build
error[E0382]: borrow of moved value: `pool`
  --> src/main.rs:40:28
   |
39 |     let bilan = pool.fermer();
   |                      -------- `pool` moved due to this method call
40 |     println!("{bilan} {}", pool.libres());
   |                            ^^^^ value borrowed here after move
   |
note: `Pool::fermer` takes ownership of the receiver `self`, which moves `pool`
  --> src/main.rs:29:15
   |
29 |     fn fermer(self) -> String {
   |               ^^^^
```

C'est une garantie que tu n'as dans aucun des trois autres langages : une connexion fermée, une transaction validée, un fichier rendu **ne peuvent plus être utilisés**, et c'est le compilateur qui le dit, pas un test ni une erreur à l'exécution. Choisis le receveur selon ce que la méthode fait : `&self` pour lire, `&mut self` pour modifier, `self` pour consommer ou transformer.

**Venant de Go :** le receveur valeur `func (p Pool)` et le receveur pointeur `func (p *Pool)` de Go correspondent à peu près à `self` et `&mut self`, mais la différence est ailleurs. En Go, le receveur valeur **copie** silencieusement, et une méthode à receveur pointeur peut modifier sans que l'appelant le voie dans la signature. En Rust, `&self` interdit de modifier, `&mut self` l'annonce, et `self` déplace au lieu de copier. Pas de règle « tous les receveurs du même genre » : chaque méthode dit ce qu'elle fait.

**Venant de Python :** `self` est explicite comme en Python, mais il porte en plus son mode d'accès. Et il n'y a pas de `__init__` : `new` est une fonction ordinaire qui construit et renvoie la struct. Rien n'empêche d'en avoir plusieurs (`Pool::new`, `Pool::depuis_url`, `Pool::avec_capacite`), et c'est ainsi qu'on remplace la surcharge, que Rust n'a pas.

### 6.3 `derive` : ce que le compilateur écrit pour toi

Les comportements de base d'un type (s'afficher, se copier, se comparer, se hacher, avoir une valeur par défaut) sont des **traits** (chapitre 7). Pour les plus courants, `#[derive(...)]` génère l'implémentation évidente, champ par champ :

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cible {
    ip: [u8; 4],
    port: u16,
}

#[derive(Debug, Clone, Default)]
struct Options {
    port: u16,
    timeout_s: u64,
    verbeux: bool,
    hotes: Vec<String>,
}

fn main() {
    let a = Cible {
        ip: [10, 0, 0, 5],
        port: 5432,
    };
    let b = a;
    println!("a toujours là : {:?}, égal à b : {}", a, a == b);

    let mut vues = HashSet::new();
    vues.insert(a);
    vues.insert(b);
    vues.insert(Cible { port: 22, ..a });
    println!("{} cibles distinctes", vues.len());

    let defaut = Options::default();
    println!("{defaut:?}");

    let prod = Options {
        port: 5432,
        hotes: vec!["db-01".to_string()],
        ..Default::default()
    };
    let copie = prod.clone();
    println!("{copie:?}");
}
```

```
$ cargo run
a toujours là : Cible { ip: [10, 0, 0, 5], port: 5432 }, égal à b : true
2 cibles distinctes
Options { port: 0, timeout_s: 0, verbeux: false, hotes: [] }
Options { port: 5432, timeout_s: 0, verbeux: false, hotes: ["db-01"] }
```

| Derive | Ce que ça permet | Condition |
|---|---|---|
| `Debug` | `{:?}` et `{:#?}` | tous les champs sont `Debug` |
| `Clone` | `.clone()`, une copie profonde explicite | tous les champs sont `Clone` |
| `Copy` | copie implicite à l'affectation, plus de déplacement | `Clone` aussi, et tous les champs `Copy` (pas de `String`, pas de `Vec`) |
| `PartialEq`, `Eq` | `==` et `!=` | tous les champs comparables ; `Eq` exclut `f64` |
| `Hash` | clé de `HashMap`, élément de `HashSet` | avec `Eq` |
| `PartialOrd`, `Ord` | `<`, `>`, `sort()`, clé de `BTreeMap` | compare les champs dans l'ordre de déclaration ; pour un enum, l'ordre des variantes |
| `Default` | `Type::default()` | tous les champs ont un défaut |

`Cible` est `Copy` : `let b = a` copie, et `a` reste utilisable, comme un entier. Elle est `Eq + Hash` : elle va dans un `HashSet`, où `a` et `b`, égales, ne comptent qu'une fois. `Options` ne peut pas être `Copy`, à cause du `Vec<String>` :

```
$ cargo build
error[E0204]: the trait `Copy` cannot be implemented for this type
 --> src/main.rs:1:24
  |
1 | #[derive(Debug, Clone, Copy)]
  |                        ^^^^
2 | struct Hote {
3 |     nom: String,
  |     ----------- this field does not implement `Copy`
```

C'est logique : copier une `String` octet par octet donnerait deux propriétaires du même tampon sur le tas, et deux libérations. `Copy` est réservé aux types qu'on peut dupliquer sans y penser ; pour les autres, `Clone` et un `.clone()` visible dans le code.

La **syntaxe de mise à jour** `..Default::default()` remplit les champs qu'on n'a pas donnés avec ceux d'une autre valeur, ici la valeur par défaut. `Cible { port: 22, ..a }` fait pareil à partir de `a`. C'est la réponse de Rust aux arguments nommés avec défaut : une struct d'options, `Default`, et on n'écrit que ce qui change.

**Piège :** à la compilation de cet exemple, le compilateur avertit `fields port, timeout_s, verbeux, and hotes are never read`, et précise que `Options` *has derived impls for the traits Clone and Debug, but these are intentionally ignored during dead code analysis*. Un champ qu'on ne lit que via `{:?}` est considéré comme mort. Dans un vrai programme, tu le lis ailleurs et l'avertissement disparaît ; dans un exemple, il est normal.

**Venant de Python :** `#[derive(Debug, Clone, PartialEq, Default)]` sur une struct, c'est `@dataclass` : le `__repr__`, le `__eq__` et les valeurs par défaut générés. `Hash` et `Eq` ensemble, c'est `@dataclass(frozen=True)` pour aller dans un `set`. La différence : rien n'est généré sans que tu le demandes.

### 6.4 L'enum : un type qui est l'un ou l'autre

Un enum déclare un type dont chaque valeur est **une** de ses variantes. Dans sa forme simple, c'est l'énumération du C. Dans sa forme complète, chaque variante porte ses propres données :

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Etat {
    Actif,
    Maintenance,
    Arrete,
}

#[derive(Debug)]
enum Evenement {
    Demarrage,
    Sonde { hote: String, ms: u32 },
    Panne(String),
    Disque { point: String, pourcent: u8 },
}

impl Evenement {
    fn gravite(&self) -> u8 {
        match self {
            Evenement::Demarrage => 0,
            Evenement::Sonde { ms, .. } if *ms > 100 => 1,
            Evenement::Sonde { .. } => 0,
            Evenement::Disque { pourcent: 90.., .. } => 2,
            Evenement::Disque { .. } => 1,
            Evenement::Panne(_) => 3,
        }
    }

    fn resume(&self) -> String {
        match self {
            Evenement::Demarrage => "démarrage".to_string(),
            Evenement::Sonde { hote, ms } => format!("{hote} répond en {ms} ms"),
            Evenement::Panne(hote) => format!("{hote} ne répond plus"),
            Evenement::Disque { point, pourcent } => format!("{point} plein à {pourcent} %"),
        }
    }
}

fn main() {
    let parc = [Etat::Actif, Etat::Maintenance, Etat::Actif, Etat::Arrete];
    let actifs = parc.iter().filter(|&&e| e == Etat::Actif).count();
    println!(
        "{actifs} actifs sur {}, le dernier est {:?}",
        parc.len(),
        parc[3]
    );
    println!("taille d'un Etat : {} octet", std::mem::size_of::<Etat>());

    let journal = vec![
        Evenement::Demarrage,
        Evenement::Sonde {
            hote: "web-01".to_string(),
            ms: 12,
        },
        Evenement::Sonde {
            hote: "db-01".to_string(),
            ms: 250,
        },
        Evenement::Disque {
            point: "/var".to_string(),
            pourcent: 93,
        },
        Evenement::Panne("cache-01".to_string()),
    ];
    for e in &journal {
        println!("[{}] {}", e.gravite(), e.resume());
    }

    if let Evenement::Panne(hote) = &journal[4] {
        println!("à redémarrer : {hote}");
    }
    let pannes = journal
        .iter()
        .filter(|e| matches!(e, Evenement::Panne(_)))
        .count();
    println!("{pannes} panne(s)");
}
```

```
$ cargo run
2 actifs sur 4, le dernier est Arrete
taille d'un Etat : 1 octet
[0] démarrage
[0] web-01 répond en 12 ms
[1] db-01 répond en 250 ms
[2] /var plein à 93 %
[3] cache-01 ne répond plus
à redémarrer : cache-01
1 panne(s)
```

- `Etat` est un enum simple, une liste de noms. Il tient sur un octet, et `Etat::Actif` n'est pas un entier qu'on pourrait confondre avec un autre.
- `Evenement` est un enum à données : `Demarrage` ne porte rien, `Panne(String)` porte un nom d'hôte comme une struct tuple, `Sonde { hote, ms }` porte des champs nommés comme une struct. Une valeur de type `Evenement` est **exactement une** de ces formes, et seulement les données de cette forme existent. Pas de champ `ms` qui traîne, vide, sur une panne.
- On en sort avec `match`, qui **déstructure** : `Evenement::Sonde { hote, ms } => ...` lie `hote` et `ms` aux données de la variante. `..` ignore les champs restants, `_` ignore une valeur. Un bras peut avoir une **garde** (`if *ms > 100`) et un motif d'intervalle (`pourcent: 90..`, 90 ou plus).
- `if let Evenement::Panne(hote) = &journal[4]` teste une seule variante quand les autres ne t'intéressent pas. `matches!(e, Evenement::Panne(_))` rend un `bool`, parfait dans un `filter`.
- `impl Evenement` attache des méthodes à l'enum comme à une struct. `match self` dans une méthode à `&self` lie des références aux données (`hote` est un `&String`).

Le compilateur vérifie que `match` couvre **toutes** les variantes. Oublie le bras `Panne` dans `resume` :

```
$ cargo build
error[E0004]: non-exhaustive patterns: `&Evenement::Panne(_)` not covered
  --> src/main.rs:29:15
   |
29 |         match self {
   |               ^^^^ pattern `&Evenement::Panne(_)` not covered
   |
note: `Evenement` defined here
...
12 |     Panne(String),
   |     ----- not covered
```

C'est la meilleure raison d'utiliser des enums. Le jour où tu ajoutes une variante `Certificat { hote, jours_restants }`, chaque `match` du programme qui ne la traite pas refuse de compiler, et la liste des erreurs est la liste des endroits à mettre à jour. Évite donc `_ => ...` sur tes propres enums : il fait taire exactement cette vérification.

**Venant de Go :** c'est le type somme que Go n'a pas. En Go, tu l'imites avec une interface et un type switch (`switch e := ev.(type)`), et le compilateur ne te dit jamais qu'un cas manque ; ou avec une struct qui a tous les champs de toutes les variantes, plus un champ `Kind`, et rien n'empêche de lire `Ms` sur une panne. Les constantes `iota` sont l'enum simple, mais un `type Etat int` accepte n'importe quel entier, y compris `Etat(42)`. En Rust, `Etat` n'a que trois valeurs possibles, et `Evenement` a ses formes et rien d'autre.

**Venant du C :** un enum à données est l'**union taguée** que tu écrivais à la main : une `struct` avec un `enum kind` et une `union` des cas possibles, plus la discipline de toujours tester `kind` avant de lire l'union. Rust range le tag et l'union pareil en mémoire, et c'est le compilateur qui tient la discipline : impossible de lire la mauvaise branche de l'union.

### 6.5 `Option<T>` : la valeur qui peut manquer

Voici sa définition, dans la bibliothèque standard :

```rust
enum Option<T> {
    None,
    Some(T),
}
```

C'est tout. Un enum générique (le `<T>` est le sujet du chapitre 7) à deux variantes : il y a une valeur, ou il n'y en a pas. `Some` et `None` sont importés partout automatiquement, d'où l'absence de `Option::` devant. Toute fonction dont le résultat peut manquer rend un `Option` : `get` sur un `Vec` ou une `HashMap`, `first`, `find`, `max`, `parse().ok()`, `split_once`. Et il faut en sortir explicitement :

```rust
use std::collections::HashMap;

fn port_de(cible: &str) -> Option<u16> {
    let (_, port) = cible.split_once(':')?;
    port.parse().ok()
}

fn port_par_defaut(service: &str) -> u16 {
    println!("  (calcul du port par défaut de {service})");
    6379
}

fn main() {
    let ports = HashMap::from([("postgres", 5432), ("ssh", 22)]);

    let pg: Option<&u16> = ports.get("postgres");
    let redis = ports.get("redis");
    println!("{pg:?} {redis:?}");

    match redis {
        Some(p) => println!("redis sur {p}"),
        None => println!("redis inconnu"),
    }
    if let Some(p) = pg {
        println!("postgres sur {p}");
    }
    let Some(ssh) = ports.get("ssh") else {
        println!("pas de ssh");
        return;
    };
    println!("ssh sur {ssh}");

    println!("unwrap_or       : {}", redis.copied().unwrap_or(6379));
    println!(
        "unwrap_or       : {}",
        pg.copied().unwrap_or(port_par_defaut("pg"))
    );
    println!(
        "unwrap_or_else  : {}",
        pg.copied().unwrap_or_else(|| port_par_defaut("pg"))
    );
    println!("map             : {:?}", pg.map(|p| p + 1000));
    println!(
        "and_then        : {:?}",
        port_de("db-01:5432").and_then(|p| p.checked_add(1000))
    );
    println!(
        "and_then (trop) : {:?}",
        port_de("db-01:65000").and_then(|p| p.checked_add(1000))
    );
    println!(
        "ok_or           : {:?}",
        port_de("db-01").ok_or("port manquant")
    );
    println!("is_some_and     : {}", pg.is_some_and(|&p| p < 1024));
    println!("filter          : {:?}", pg.filter(|&&p| p > 1024));
    println!(
        "? sur Option    : {:?} {:?}",
        port_de("db-01:5432"),
        port_de("db-01:pg")
    );

    let mut nom: Option<String> = Some("db-01".to_string());
    let longueur = nom.as_ref().map(|s| s.len());
    println!("as_ref          : {longueur:?}, nom toujours là : {nom:?}");
    if let Some(s) = nom.as_mut() {
        s.push_str(".prod");
    }
    let pris = nom.take();
    println!("take            : {pris:?}, reste {nom:?}");
}
```

```
$ cargo run
Some(5432) None
redis inconnu
postgres sur 5432
ssh sur 22
unwrap_or       : 6379
  (calcul du port par défaut de pg)
unwrap_or       : 5432
unwrap_or_else  : 5432
map             : Some(6432)
and_then        : Some(6432)
and_then (trop) : None
ok_or           : Err("port manquant")
is_some_and     : false
filter          : Some(5432)
? sur Option    : Some(5432) None
as_ref          : Some(5), nom toujours là : Some("db-01")
take            : Some("db-01.prod"), reste None
```

Les trois façons de sortir, du plus général au plus court :

- `match` traite les deux cas. `if let Some(p) = ...` traite le cas présent et ignore l'autre.
- `let Some(ssh) = ... else { ...; return; };` (*let-else*) : si la valeur manque, on exécute le bloc, qui doit sortir de la fonction (`return`, `continue`, `break`, ou un panic). Sinon, `ssh` est disponible pour la suite **sans indentation**. C'est le retour anticipé de Go, `if !ok { return }`, en une instruction.
- Le **`?`** sur un `Option`, dans une fonction qui rend elle-même un `Option` : `cible.split_once(':')?` rend `None` tout de suite s'il n'y a pas de `:`, et sinon donne la paire. `port_de` s'écrit en deux lignes, sans un seul `if`. Le chapitre 8 fait la même chose avec `Result`, et c'est là que `?` devient indispensable.

Et les méthodes, qui évitent la plupart des `match` :

- `unwrap_or(v)` rend la valeur ou `v`. `unwrap_or_else(|| ...)` appelle la fermeture seulement si la valeur manque. Regarde la sortie : `unwrap_or(port_par_defaut("pg"))` a **calculé** le défaut alors que la valeur était là, parce que les arguments d'une fonction sont évalués avant l'appel. Pour un défaut qui coûte (un appel réseau, une allocation), `unwrap_or_else`. `unwrap_or_default()` prend le `Default` du type.
- `map(f)` transforme la valeur si elle est là et laisse `None` tranquille. `and_then(f)` enchaîne une opération qui peut elle-même échouer (`checked_add` rend `None` en cas de débordement), sans se retrouver avec un `Option<Option<u16>>`.
- `ok_or(erreur)` transforme un `Option` en `Result` : `None` devient `Err(erreur)`. C'est le pont vers le chapitre 8.
- `is_some_and(f)` teste la valeur si elle est là ; `filter(f)` la garde si elle vérifie la condition.
- `copied()` transforme un `Option<&u16>` en `Option<u16>`, comme pour les itérateurs.

Restent trois méthodes qui parlent de possession. `map` prend l'`Option` **par valeur** : sur un `Option<String>`, il déplace la chaîne, et l'option n'est plus utilisable.

```
$ cargo build
error[E0382]: borrow of moved value: `nom`
 --> src/main.rs:4:29
  |
2 |     let nom: Option<String> = Some("db-01".to_string());
  |         --- move occurs because `nom` has type `Option<String>`, which does not implement the `Copy` trait
3 |     let longueur = nom.map(|s| s.len());
  |                    --- ---------------- `nom` moved due to this method call
  |                    |
  |                    help: consider calling `.as_ref()` or `.as_mut()` to borrow the type's contents
4 |     println!("{longueur:?} {nom:?}");
  |                             ^^^ value borrowed here after move
```

Le compilateur donne la solution : `as_ref()` transforme un `&Option<String>` en `Option<&String>`, on travaille sur une référence, et l'original reste. `as_mut()` donne un `Option<&mut String>` pour modifier sur place. `take()` sort la valeur et laisse `None` à sa place : c'est ainsi qu'on récupère le contenu d'un champ `Option` d'une struct qu'on n'a qu'en `&mut` (une connexion qu'on ferme, un fil qu'on joint).

**Et `unwrap()` ?** Il rend la valeur, et panique sur `None`. `expect("message")` fait pareil avec ton message. Ils sont à leur place dans un test, dans un exemple, ou quand un `None` serait un bug que tu préfères voir tout de suite. Ailleurs, chaque `unwrap` est un plantage potentiel en production. Le chapitre 8 précise la règle.

### 6.6 Pourquoi pas de `null`

En C, en Go, en Java, une référence peut être nulle, et rien dans le type ne le dit. Un `*Hote` en Go peut être `nil` ; une fonction qui le reçoit doit le tester, ou espérer. Tony Hoare, qui a inventé la référence nulle en 1965, l'appelle son « erreur à un milliard de dollars ».

En Rust, une référence `&Hote` **n'est jamais nulle**. Si une valeur peut manquer, son type est `Option<&Hote>`, et le compilateur t'oblige à traiter `None` avant de t'en servir. Le test que tu oubliais n'est plus oubliable, et inversement, une fonction qui prend un `&Hote` n'a pas à se demander si on lui a passé du vide.

Et ça ne coûte rien :

```rust
use std::mem::size_of;

fn main() {
    println!("&u64          : {} octets", size_of::<&u64>());
    println!("Option<&u64>  : {} octets", size_of::<Option<&u64>>());
    println!("Box<u64>      : {} octets", size_of::<Box<u64>>());
    println!("Option<Box>   : {} octets", size_of::<Option<Box<u64>>>());
    println!("u64           : {} octets", size_of::<u64>());
    println!("Option<u64>   : {} octets", size_of::<Option<u64>>());
}
```

```
$ cargo run
&u64          : 8 octets
Option<&u64>  : 8 octets
Box<u64>      : 8 octets
Option<Box>   : 8 octets
u64           : 8 octets
Option<u64>   : 16 octets
```

Une référence ou un `Box` ne peuvent pas valoir zéro, donc le compilateur utilise l'adresse zéro pour représenter `None` : `Option<&T>` a la même taille qu'un pointeur, et le test `is_some()` est la comparaison avec zéro que tu aurais écrite en C. Pour un `u64`, toutes les valeurs sont possibles : il faut un tag à côté, et l'alignement sur 8 octets porte le total à 16.

**Venant de Go :** `Option` remplace trois idiomes Go à la fois. Le pointeur qui peut être `nil` (`*Hote`), la valeur zéro qui veut dire « absent » (un port `0`, une chaîne vide, avec l'ambiguïté quand zéro est une vraie valeur), et le couple `v, ok := m[k]`. Un seul type, et le compilateur vérifie.

**Venant de Python :** `None` existe en Python, et n'importe quelle variable peut le valoir. `Optional[int]` dans les annotations dit la même chose qu'`Option<i32>`, mais c'est mypy qui le vérifie, si tu le lances. Ici, c'est le compilateur, toujours.

### 6.7 `Display` : s'afficher pour un humain

`{:?}` (`Debug`) est pour le développeur : il montre la structure. `{}` est pour l'utilisateur, et demande d'implémenter le trait `std::fmt::Display` à la main, parce que le compilateur ne peut pas deviner ce que tu veux afficher :

```rust
use std::fmt;

struct Sonde {
    cible: String,
    ms: u32,
    ok: bool,
}

impl fmt::Display for Sonde {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let etat = if self.ok { "OK" } else { "KO" };
        write!(f, "{:<16} {} en {} ms", self.cible, etat, self.ms)
    }
}

struct Octets(u64);

impl fmt::Display for Octets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let unites = ["o", "Kio", "Mio", "Gio", "Tio"];
        let mut valeur = self.0 as f64;
        let mut i = 0;
        while valeur >= 1024.0 && i < unites.len() - 1 {
            valeur /= 1024.0;
            i += 1;
        }
        let texte = if i == 0 {
            format!("{} {}", self.0, unites[0])
        } else {
            format!("{:.1} {}", valeur, unites[i])
        };
        f.pad(&texte)
    }
}

fn main() {
    let s = Sonde {
        cible: "10.0.0.5:5432".to_string(),
        ms: 3,
        ok: true,
    };
    println!("{s}");
    let texte = s.to_string();
    println!("{} octets", texte.len());
    println!("[{:>30}]", s);

    for n in [512, 1536, 5_000_000, 3 * 1024 * 1024 * 1024] {
        println!("{:>10} -> [{:>8}]", n, Octets(n));
    }
}
```

```
$ cargo run
10.0.0.5:5432    OK en 3 ms
27 octets
[10.0.0.5:5432    OK en 3 ms]
       512 -> [   512 o]
      1536 -> [ 1.5 Kio]
   5000000 -> [ 4.8 Mio]
3221225472 -> [ 3.0 Gio]
```

- `impl fmt::Display for Sonde` implémente un trait (chapitre 7 pour le détail) : une seule méthode, `fmt`, qui reçoit un `Formatter` et y écrit avec `write!`, le `format!` qui écrit ailleurs qu'en mémoire. Elle rend un `fmt::Result`, que `write!` fournit.
- Implémenter `Display` donne **gratuitement** `to_string()` : toute valeur affichable sait se convertir en `String`.
- **Piège :** `[{:>30}]` n'a rien aligné sur `Sonde`. La largeur demandée par l'appelant est rangée dans le `Formatter`, et `write!` l'ignore. Si ton type doit respecter `{:>8}`, construis le texte puis passe-le à `f.pad(&texte)`, comme `Octets` : là, la colonne est alignée.

**Venant de Python :** `Display` est `__str__`, `Debug` est `__repr__`. **Venant de Go :** `Display` est `String() string` de `fmt.Stringer`, et `Debug` ressemble à ce que `%+v` affiche. Le piège Go de la récursion infinie (`Sprintf("%v", s)` dans `String()`) n'existe pas sous cette forme : `write!(f, "{}", self)` dans `fmt` compile sans un avertissement, et s'arrête à l'exécution sur `thread 'main' has overflowed its stack`. Formate les champs, pas `self`.

### 6.8 Visibilité : `pub` et `pub(crate)`

Tout est **privé par défaut** : un champ, une méthode, une struct ne sont visibles que dans le module où ils sont déclarés (le chapitre 9 détaille les modules ; ici, `mod banque { ... }` en déclare un dans le même fichier). `pub` rend visible partout, `pub(crate)` dans toute la crate mais pas au-delà.

```rust
mod banque {
    pub struct Compte {
        pub titulaire: String,
        pub(crate) devise: &'static str,
        solde_centimes: i64,
    }

    impl Compte {
        pub fn new(titulaire: &str) -> Self {
            Compte {
                titulaire: titulaire.to_string(),
                devise: "EUR",
                solde_centimes: 0,
            }
        }

        pub fn deposer(&mut self, centimes: i64) {
            if centimes > 0 {
                self.solde_centimes += centimes;
            }
        }

        pub fn solde(&self) -> i64 {
            self.solde_centimes
        }
    }
}

use banque::Compte;

fn main() {
    let mut c = Compte::new("Gilles");
    c.deposer(1250);
    c.deposer(-100_000);
    println!("{} : {} centimes en {}", c.titulaire, c.solde(), c.devise);
}
```

```
$ cargo run
Gilles : 1250 centimes en EUR
```

`solde_centimes` est privé : hors du module, personne ne peut le modifier autrement que par `deposer`, qui refuse les montants négatifs. Essaie quand même :

```
$ cargo build
error[E0616]: field `solde_centimes` of struct `Compte` is private
  --> src/main.rs:34:7
   |
34 |     c.solde_centimes = 1_000_000_00;
   |       ^^^^^^^^^^^^^^ private field
```

Et un champ privé interdit aussi de **construire** la struct avec la syntaxe littérale hors du module (`error[E0451]: field solde_centimes of struct Compte is private`). C'est volontaire : la seule façon d'obtenir un `Compte` est `Compte::new`, qui garantit un état de départ valide. Une struct dont tous les champs sont `pub` est un simple sac de données ; une struct à champs privés est un type qui protège un **invariant** (un solde qui ne descend que par des retraits vérifiés). Choisis selon ce que tu veux garantir.

**Venant de Go :** la majuscule en Go fait le travail de `pub`, et la frontière est le paquet. Même idée, autre syntaxe : `pub` au lieu de `Solde`, module au lieu de paquet. `pub(crate)` n'a pas d'équivalent direct (le plus proche est un paquet `internal/`).

### 6.9 Méthodes qui consomment `self` : le builder

Une méthode à receveur `self` qui rend `Self` permet d'enchaîner les appels en passant la valeur de main en main. C'est le motif **builder**, qu'utilisent `reqwest`, `clap`, `tokio` et la moitié de l'écosystème :

```rust
use std::time::Duration;

struct Client {
    hote: String,
    port: u16,
    timeout: Duration,
    tls: bool,
}

struct ClientBuilder {
    hote: String,
    port: u16,
    timeout: Duration,
    tls: bool,
}

impl ClientBuilder {
    fn new(hote: &str) -> Self {
        ClientBuilder {
            hote: hote.to_string(),
            port: 5432,
            timeout: Duration::from_secs(5),
            tls: false,
        }
    }

    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn tls(mut self, actif: bool) -> Self {
        self.tls = actif;
        self
    }

    fn construire(self) -> Client {
        Client {
            hote: self.hote,
            port: self.port,
            timeout: self.timeout,
            tls: self.tls,
        }
    }
}

fn main() {
    let simple = ClientBuilder::new("db-01").construire();
    let prod = ClientBuilder::new("db-02.prod")
        .port(6432)
        .tls(true)
        .timeout(Duration::from_millis(1500))
        .construire();
    for c in [simple, prod] {
        println!(
            "{}:{} tls={} timeout={:?}",
            c.hote, c.port, c.tls, c.timeout
        );
    }
}
```

```
$ cargo run
db-01:5432 tls=false timeout=5s
db-02.prod:6432 tls=true timeout=1.5s
```

Chaque méthode prend le builder par valeur (`mut self`, pour pouvoir modifier ce qu'on vient de recevoir), change un champ et le rend. `construire(self)` le consomme et produit le `Client`. Aucune copie : la même valeur est déplacée de méthode en méthode. On ne donne que ce qui change, dans l'ordre qu'on veut, et le builder ne peut pas être réutilisé par erreur après `construire`. Quand une struct a plus de trois ou quatre options, c'est la forme la plus lisible.

**Venant de Go :** c'est le motif des options fonctionnelles (`NewClient(hote, WithPort(6432), WithTLS())`) écrit autrement. **Pas d'héritage non plus** : comme en Go, Rust n'a ni classes ni héritage. Le partage de comportement passe par les traits, qui sont le sujet du chapitre suivant, et le partage de données par la composition (une struct qui contient une autre struct).

### 6.10 Pour le labo

Le [labo 06](../labs/06-compte-bancaire/README.md) modélise une petite banque : un enum `Operation` (dépôt, retrait, virement émis, virement reçu), un enum `ErreurBanque` avec `Display` dont les messages sont testés, une struct `Compte` avec `new`, `deposer(&mut self)`, `retirer(&mut self)` et `solde(&self)`, et une `Banque` qui range les comptes dans une `HashMap`. Le cœur du labo est `virer(de, vers, montant)` : il faut modifier deux comptes de la même map, donc deux `&mut` en même temps, et le vérificateur d'emprunts va te dire non. La solution explique comment s'y prendre proprement, et le relevé de compte, au format exact, te fera écrire du `Display` et du `match`.

### À retenir

- Une struct se construit avec tous ses champs ; la mutabilité est celle de la variable. Struct tuple pour un *newtype*, struct unité pour un marqueur.
- Le receveur d'une méthode dit ce qu'elle fait : `&self` lit, `&mut self` modifie, `self` consomme. Le compilateur interdit d'utiliser une valeur après une méthode qui l'a consommée.
- `new` est une fonction associée ordinaire ; plusieurs constructeurs remplacent la surcharge.
- `derive` écrit `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Ord`, `Default` ; `Copy` seulement si tous les champs le sont. `..Default::default()` remplit le reste.
- Un enum est l'un ou l'autre de ses variantes, chacune avec ses données ; `match` est exhaustif et le compilateur liste les cas oubliés. Évite `_` sur tes propres enums.
- `Option<T>` est un enum `Some(T)` ou `None`, et remplace `null`, la valeur zéro et `v, ok`. Sors-en avec `match`, `if let`, `let else` ou `?`.
- `unwrap_or_else` plutôt qu'`unwrap_or` pour un défaut qui coûte ; `as_ref` pour ne pas déplacer, `take` pour sortir d'un `&mut`, `ok_or` pour passer à `Result`.
- `Display` à la main pour `{}` et `to_string()` ; `f.pad` si la largeur doit être respectée.
- Tout est privé par défaut ; des champs privés et un `new` protègent un invariant.

---

← [5. Collections et itérateurs](05-collections-et-iterateurs.md) · [Sommaire](../README.md) · [7. Traits et génériques](07-traits-et-generiques.md) →
