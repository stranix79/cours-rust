# 7. Traits et génériques

*Rust de zéro à la prod : chapitre 7 sur 16.* ← [6. Structs, enums, méthodes, Option](06-structs-enums-option.md) · [Sommaire](../README.md) · [8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic](08-erreurs.md) →

Le chapitre 6 t'a donné des types à toi : des structs, des enums, des méthodes. Il manque une chose : écrire une fonction qui accepte **plusieurs** types, pourvu qu'ils sachent faire quelque chose. Une fonction qui vérifie « n'importe quel contrôle de supervision », qu'il s'agisse d'un disque, d'un port ou d'un certificat. Une fonction qui trouve le plus grand élément d'une liste, que ce soient des entiers, des flottants ou des versions. En Go, tu écrivais une interface ; en Python, tu ne déclarais rien et tu croisais les doigts.

Rust répond avec deux outils qui marchent ensemble. Le **trait** déclare un comportement (« sait se vérifier », « sait s'afficher », « sait se comparer »). Le **générique** écrit une fonction pour tout type qui a ce comportement. Et l'**objet trait** (`dyn`) range des types différents dans une même collection. C'est le chapitre qui explique enfin les `derive`, le `Display` et le `FromIterator` que tu utilises depuis deux chapitres sans les nommer.

### 7.1 Définir un trait, l'implémenter

Un trait est une liste de méthodes qu'un type promet de fournir. On le déclare avec `trait`, on l'implémente avec `impl Trait for Type`. Une méthode peut avoir un corps **par défaut**, que chaque type garde ou remplace :

```rust
trait Controle {
    fn nom(&self) -> String;
    fn verifier(&self) -> bool;

    fn rapport(&self) -> String {
        let etat = if self.verifier() { "OK" } else { "KO" };
        format!("[{etat}] {}", self.nom())
    }
}

struct Disque {
    point: String,
    pourcent: u8,
}

struct Certificat {
    domaine: String,
    jours_restants: i32,
}

impl Controle for Disque {
    fn nom(&self) -> String {
        format!("disque {}", self.point)
    }

    fn verifier(&self) -> bool {
        self.pourcent < 90
    }
}

impl Controle for Certificat {
    fn nom(&self) -> String {
        format!("certificat {}", self.domaine)
    }

    fn verifier(&self) -> bool {
        self.jours_restants > 14
    }

    fn rapport(&self) -> String {
        let etat = if self.verifier() { "OK" } else { "KO" };
        format!(
            "[{etat}] {} expire dans {} jours",
            self.nom(),
            self.jours_restants
        )
    }
}

fn main() {
    let var = Disque {
        point: "/var".to_string(),
        pourcent: 93,
    };
    let web = Certificat {
        domaine: "stranix.net".to_string(),
        jours_restants: 41,
    };
    println!("{}", var.rapport());
    println!("{}", web.rapport());
    println!("{} {}", var.verifier(), web.nom());
}
```

```
$ cargo run
[KO] disque /var
[OK] certificat stranix.net expire dans 41 jours
false certificat stranix.net
```

- `trait Controle` déclare trois méthodes. `nom` et `verifier` n'ont pas de corps : chaque type **doit** les écrire. `rapport` en a un : c'est une méthode par défaut, construite sur les deux autres.
- `impl Controle for Disque` fournit les deux méthodes obligatoires et garde le `rapport` par défaut. `Certificat` remplace `rapport` pour ajouter les jours restants.
- Une méthode par défaut peut appeler les autres méthodes du trait (`self.verifier()`, `self.nom()`), même celles qui n'ont pas de corps. C'est ce qui rend les traits puissants : tu écris deux méthodes, tu en reçois dix. `Iterator` en est l'exemple extrême (section 7.6).

Oublie une méthode obligatoire, et le compilateur te le dit à l'endroit exact :

```
$ cargo build
error[E0046]: not all trait items implemented, missing: `verifier`
  --> src/main.rs:21:1
   |
 3 |     fn verifier(&self) -> bool;
   |     --------------------------- `verifier` from trait
...
21 | impl Controle for Disque {
   | ^^^^^^^^^^^^^^^^^^^^^^^^ missing `verifier` in implementation
```

Pour appeler une méthode d'un trait, le trait doit être **en portée**. Ici il est déclaré dans le même fichier, donc tout va bien. Quand il vient d'une autre crate ou d'un autre module, il faut un `use` (`use std::fmt::Write;` pour écrire dans une `String` avec `write!`, `use std::io::Read;` pour `read_to_string`). Le compilateur, si tu l'oublies, te propose le `use` à ajouter.

**Venant de Go :** une interface Go est satisfaite **implicitement** : un type qui a les bonnes méthodes est une `Forme`, sans le dire. En Rust, c'est **explicite** : sans `impl Controle for Disque`, `Disque` n'est pas un `Controle`, même avec les bonnes méthodes. C'est plus verbeux, et c'est voulu. L'intention est écrite, une faute de frappe dans un nom de méthode est une erreur au lieu d'un type qui ne satisfait plus rien en silence, et surtout tu peux implémenter **ton** trait pour un type que tu n'as pas écrit : `impl Controle for u16` est légal, alors qu'en Go tu ne peux pas ajouter de méthode à `int`. Les méthodes par défaut n'existent pas en Go ; on les imite en embarquant une struct.

**Venant de Python :** le *duck typing* (« si ça cancane, c'est un canard ») est vérifié à l'exécution, au moment où la méthode manque. Un trait, c'est une `abc.ABC` avec des `@abstractmethod`, ou un `typing.Protocol`, mais vérifié par le compilateur, toujours, sans lancer mypy.

### 7.2 Les génériques et leurs bornes

Une fonction générique a un ou plusieurs **paramètres de type**, entre chevrons. Seule, `fn plus_grand<T>(a: T, b: T) -> T` ne peut presque rien faire de `a` et `b` : le compilateur ne sait rien de `T`. Une **borne** (`T: PartialOrd`) dit ce que `T` doit savoir faire, et du coup ce que la fonction a le droit d'utiliser :

```rust
use std::fmt::Display;

fn plus_grand<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn le_plus_grand<T>(valeurs: &[T]) -> Option<&T>
where
    T: PartialOrd,
{
    let mut reste = valeurs.iter();
    let mut max = reste.next()?;
    for v in reste {
        if v > max {
            max = v;
        }
    }
    Some(max)
}

fn alerte<T>(nom: &str, valeur: T, seuil: T) -> Option<String>
where
    T: PartialOrd + Display,
{
    if valeur > seuil {
        Some(format!("{nom} : {valeur} dépasse {seuil}"))
    } else {
        None
    }
}

fn main() {
    println!("{}", plus_grand(3, 7));
    println!("{}", plus_grand(2.5, 1.0));
    println!("{}", plus_grand("db-01", "web-01"));
    println!("{:?}", le_plus_grand(&[12, 80, 15]));
    println!("{:?}", le_plus_grand::<u32>(&[]));
    println!("{:?}", alerte("charge", 0.93, 0.8));
    println!("{:?}", alerte("connexions", 40, 100));
}
```

```
$ cargo run
7
2.5
web-01
Some(80)
None
Some("charge : 0.93 dépasse 0.8")
None
```

- `fn plus_grand<T: PartialOrd>(a: T, b: T) -> T` : pour tout type `T` comparable avec `<` et `>`. Elle marche sur des entiers, des flottants, des `&str` (ordre alphabétique). `PartialOrd` et pas `Ord`, pour accepter les flottants (le `NaN` du chapitre 5).
- `where` déplace les bornes après la signature. Même sens, plus lisible dès qu'il y en a plusieurs. `T: PartialOrd + Display` exige les deux : comparable **et** affichable, puisque `alerte` fait les deux.
- `le_plus_grand::<u32>(&[])` : avec une tranche vide, le compilateur ne peut pas deviner `T`, alors on le lui donne en *turbofish*, comme pour `collect` au chapitre 5.
- Les génériques ne sont pas réservés aux fonctions : `Vec<T>`, `Option<T>` et `HashMap<K, V>` sont des types génériques, et ta struct peut l'être aussi (`struct Paire<T> { a: T, b: T }`).

Sans la borne, la fonction ne compile pas, et le message est un modèle du genre :

```
$ cargo build
error[E0369]: binary operation `>` cannot be applied to type `T`
 --> src/main.rs:2:10
  |
2 |     if a > b { a } else { b }
  |        - ^ - T
  |        |
  |        T
  |
help: consider restricting type parameter `T` with trait `PartialOrd`
  |
1 | fn plus_grand<T: std::cmp::PartialOrd>(a: T, b: T) -> T {
  |                ++++++++++++++++++++++
```

C'est le contrat dans les deux sens. L'appelant sait ce qu'il doit fournir (un type comparable) ; la fonction sait ce qu'elle a le droit de faire (comparer, rien d'autre). Si tu appelles `plus_grand` avec un type qui n'est pas `PartialOrd`, l'erreur est **à l'appel**, avec le nom du trait manquant, et pas trois niveaux plus bas dans le corps de la fonction comme avec les templates C++.

**Venant de Go :** `func PlusGrand[T cmp.Ordered](a, b T) T` est la même idée, avec deux différences. Une contrainte Go est une interface, souvent une liste de types (`~int | ~float64`) ; une borne Rust est un trait, que n'importe quel type peut implémenter, y compris les tiens. Et Go compile un générique en partageant le code entre types de même forme, avec un dictionnaire caché ; Rust produit une copie spécialisée par type (section 7.5), aussi rapide que si tu l'avais écrite à la main.

### 7.3 `impl Trait` : en argument et en retour

Écrire `<T: Trait>` pour un paramètre qui n'apparaît qu'une fois est lourd. `impl Trait` est le raccourci : « un type, n'importe lequel, qui implémente ce trait ». En retour, il veut dire « un type précis que je ne nomme pas » :

```rust
use std::fmt::Display;

fn liste(elements: impl IntoIterator<Item = impl Display>) -> String {
    elements
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn ports_de(base: u16) -> impl Iterator<Item = u16> {
    (0..3).map(move |i| base + i)
}

fn seuil(max: u8) -> impl Fn(u8) -> bool {
    move |pourcent| pourcent < max
}

fn main() {
    println!("{}", liste([22, 80, 443]));
    println!("{}", liste(vec!["web-01", "db-01"]));
    println!("{}", liste(ports_de(5432)));
    let disque_ok = seuil(90);
    println!("{} {}", disque_ok(42), disque_ok(97));
}
```

```
$ cargo run
22, 80, 443
web-01, db-01
5432, 5433, 5434
true false
```

- `liste(elements: impl IntoIterator<Item = impl Display>)` accepte un tableau, un `Vec`, un itérateur, n'importe quoi qu'on peut parcourir et dont les éléments s'affichent. C'est exactement `fn liste<I, D>(elements: I) where I: IntoIterator<Item = D>, D: Display`, en une ligne.
- `ports_de` renvoie `impl Iterator<Item = u16>`. Le vrai type est un `Map<Range<u16>, {closure}>`, que tu ne peux même pas écrire (une fermeture n'a pas de nom). `impl Iterator` dit à l'appelant ce qu'il peut faire avec, et cache le reste. C'est la façon normale de renvoyer un itérateur.
- `seuil` renvoie une fermeture : `impl Fn(u8) -> bool`. Le `move` du chapitre 5 est obligatoire, `max` doit partir avec elle.

**Piège :** `impl Trait` en retour désigne **un seul** type concret. Une fonction qui renverrait tantôt un intervalle, tantôt un vecteur ne compile pas :

```
$ cargo build
error[E0308]: `if` and `else` have incompatible types
 --> src/main.rs:5:9
  |
2 | /     if tous {
3 | |         1..=1024
  | |         -------- expected because of this
4 | |     } else {
5 | |         vec![22, 443].into_iter()
  | |         ^^^^^^^^^^^^^^^^^^^^^^^^^ expected `RangeInclusive<{integer}>`, found `IntoIter<{integer}>`
6 | |     }
  | |_____- `if` and `else` have incompatible types
  |
```

Le message complet propose la solution : `Box<dyn Iterator<Item = u16>>`. Quand le type change selon le cas, il faut un objet trait. C'est la section suivante.

### 7.4 Les objets trait : `dyn`

Un `Vec<T>` contient des éléments d'**un** type. Pour ranger dans la même liste un disque, un port et un certificat, il faut un type commun. C'est l'**objet trait** : `dyn Controle`, « une valeur de type inconnu à la compilation, qui implémente `Controle` ». Comme sa taille est inconnue (un `Disque` et un `Port` ne font pas la même taille), on le manipule toujours derrière un pointeur : `&dyn Controle` pour l'emprunter, `Box<dyn Controle>` pour le posséder.

```rust
trait Controle {
    fn nom(&self) -> String;
    fn verifier(&self) -> bool;

    fn rapport(&self) -> String {
        let etat = if self.verifier() { "OK" } else { "KO" };
        format!("[{etat}] {}", self.nom())
    }
}

struct Disque {
    point: String,
    pourcent: u8,
}

struct Port {
    hote: String,
    port: u16,
    ouvert: bool,
}

impl Controle for Disque {
    fn nom(&self) -> String {
        format!("disque {}", self.point)
    }
    fn verifier(&self) -> bool {
        self.pourcent < 90
    }
}

impl Controle for Port {
    fn nom(&self) -> String {
        format!("port {}:{}", self.hote, self.port)
    }
    fn verifier(&self) -> bool {
        self.ouvert
    }
}

fn afficher(c: &dyn Controle) {
    println!("  -> {}", c.rapport());
}

fn depuis_config(ligne: &str) -> Option<Box<dyn Controle>> {
    let (genre, cible) = ligne.split_once(' ')?;
    match genre {
        "disque" => Some(Box::new(Disque {
            point: cible.to_string(),
            pourcent: 0,
        })),
        "port" => {
            let (hote, port) = cible.split_once(':')?;
            Some(Box::new(Port {
                hote: hote.to_string(),
                port: port.parse().ok()?,
                ouvert: false,
            }))
        }
        _ => None,
    }
}

fn main() {
    let controles: Vec<Box<dyn Controle>> = vec![
        Box::new(Disque {
            point: "/".to_string(),
            pourcent: 41,
        }),
        Box::new(Port {
            hote: "db-01".to_string(),
            port: 5432,
            ouvert: true,
        }),
        Box::new(Disque {
            point: "/var".to_string(),
            pourcent: 93,
        }),
    ];
    for c in &controles {
        println!("{}", c.rapport());
    }
    let ko = controles.iter().filter(|c| !c.verifier()).count();
    println!("{ko} en échec sur {}", controles.len());

    afficher(controles[1].as_ref());
    let local = Port {
        hote: "localhost".to_string(),
        port: 22,
        ouvert: true,
    };
    afficher(&local);

    let lus: Vec<Box<dyn Controle>> = ["disque /home", "port web-01:443", "dns x"]
        .into_iter()
        .filter_map(depuis_config)
        .collect();
    println!("{} contrôles lus : {}", lus.len(), lus[1].nom());
}
```

```
$ cargo run
[OK] disque /
[OK] port db-01:5432
[KO] disque /var
1 en échec sur 3
  -> [OK] port db-01:5432
  -> [OK] port localhost:22
2 contrôles lus : port web-01:443
```

- `Vec<Box<dyn Controle>>` : chaque `Box::new(...)` alloue la valeur sur le tas et la convertit en `Box<dyn Controle>`. La liste ne contient que des pointeurs, tous de la même taille. Le labo 07 fait exactement ça avec un `Vec<Box<dyn Forme>>`.
- `c.rapport()` appelle la bonne méthode pour chaque élément, `Disque::rapport` ou `Port::rapport`, choisie **à l'exécution**.
- `fn afficher(c: &dyn Controle)` emprunte n'importe quel contrôle. `controles[1].as_ref()` transforme un `&Box<dyn Controle>` en `&dyn Controle` ; `&local` convertit un `&Port` tout seul.
- `depuis_config` renvoie un `Option<Box<dyn Controle>>` dont le type concret dépend du texte lu. C'est le cas d'école du `dyn` : on ne sait qu'à l'exécution ce qu'on va construire, comme un système de plugins ou une configuration qui liste des vérifications.

Sans `Box<dyn ...>`, le vecteur prend le type de son premier élément, et refuse le deuxième :

```
$ cargo build
error[E0308]: mismatched types
  --> src/main.rs:69:9
   |
69 | /         Port {
70 | |             hote: "db-01".to_string(),
71 | |             port: 5432,
72 | |             ouvert: true,
73 | |         },
   | |_________^ expected `Disque`, found `Port`
```

**Venant de Go :** `[]Forme{Rectangle{3, 4}, Cercle{1}}` est le `Vec<Box<dyn Forme>>` de Go. Une valeur d'interface Go est un objet trait, et Go met tout derrière ce mécanisme ; Rust te le fait écrire, parce qu'il a un coût (section suivante). Et le *type switch* (`switch f := v.(type)`) ? Si tu as besoin de savoir quel type concret se cache derrière un `dyn`, c'est presque toujours que tu voulais un **enum** (chapitre 6) : un ensemble fermé de cas, un `match` exhaustif. Règle simple : ensemble de types **fermé** et connu (les événements de ton programme), un enum ; ensemble **ouvert** (n'importe qui peut ajouter un type, un plugin, un test qui fournit un faux), un trait.

### 7.5 Génériques ou `dyn` : comment choisir

Les deux écrivent « n'importe quel type qui implémente `Controle` ». Ils ne compilent pas pareil.

```rust
use std::mem::size_of;

trait Controle {
    fn verifier(&self) -> bool;
}

struct Disque {
    pourcent: u8,
}

impl Controle for Disque {
    fn verifier(&self) -> bool {
        self.pourcent < 90
    }
}

fn tous_ok_generique<T: Controle>(controles: &[T]) -> bool {
    controles.iter().all(|c| c.verifier())
}

fn tous_ok_dyn(controles: &[Box<dyn Controle>]) -> bool {
    controles.iter().all(|c| c.verifier())
}

fn main() {
    println!("&Disque           : {} octets", size_of::<&Disque>());
    println!("&dyn Controle     : {} octets", size_of::<&dyn Controle>());
    println!(
        "Box<dyn Controle> : {} octets",
        size_of::<Box<dyn Controle>>()
    );

    let disques = [Disque { pourcent: 40 }, Disque { pourcent: 95 }];
    let boites: Vec<Box<dyn Controle>> = vec![Box::new(Disque { pourcent: 40 })];
    println!("{} {}", tous_ok_generique(&disques), tous_ok_dyn(&boites));
}
```

```
$ cargo run
&Disque           : 8 octets
&dyn Controle     : 16 octets
Box<dyn Controle> : 16 octets
false true
```

- **Générique** (`tous_ok_generique<T: Controle>`) : le compilateur produit une copie de la fonction **par type utilisé**, où `T` est remplacé par `Disque`. C'est la *monomorphisation*. L'appel à `verifier` est direct, souvent intégré dans la boucle (*inlining*). Aucun coût à l'exécution ; le prix est payé à la compilation (plus long) et dans la taille du binaire (une copie par type). Et une tranche `&[T]` ne contient qu'un seul type à la fois.
- **`dyn`** (`tous_ok_dyn`) : une seule copie de la fonction. Un `&dyn Controle` fait 16 octets : un pointeur vers la valeur, et un pointeur vers une **table virtuelle** (*vtable*), qui contient l'adresse de chaque méthode pour ce type. Chaque appel passe par cette table : une indirection, pas d'*inlining*. En échange, des types différents dans la même collection.

Le choix, dans l'ordre :

1. **Générique par défaut.** C'est ce que fait la bibliothèque standard partout (`Iterator::map`, `Vec<T>`, `HashMap<K, V>`), et c'est ce que tu écris quand une fonction travaille sur un type à la fois.
2. **`dyn` quand les types sont mélangés** dans une même collection, ou choisis à l'exécution (configuration, plugins), ou quand une API publique doit rester simple (`Box<dyn Error>`, au chapitre 8, est l'exemple que tu croiseras partout).
3. Le coût de `dyn` est réel mais petit : une indirection de pointeur par appel. Dans un programme qui fait des entrées-sorties, il est invisible. Ne choisis pas un générique pour la performance avant d'avoir mesuré.

Tous les traits ne peuvent pas devenir des objets trait. Une méthode qui renvoie `Self`, ou qui a ses propres paramètres génériques, ne peut pas aller dans une table virtuelle (quelle taille réserver pour un `Self` inconnu ?) :

```
$ cargo build
error[E0038]: the trait `Dupliquer` is not dyn compatible
  --> src/main.rs:14:20
   |
14 |     let d: Box<dyn Dupliquer> = Box::new(Disque);
   |                    ^^^^^^^^^ `Dupliquer` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> src/main.rs:2:28
   |
 1 | trait Dupliquer {
   |       --------- this trait is not dyn compatible...
 2 |     fn dupliquer(&self) -> Self;
   |                            ^^^^ ...because method `dupliquer` references the `Self` type in its return type
   = help: consider moving `dupliquer` to another trait
   = help: only type `Disque` implements `Dupliquer`; consider using it directly instead.
```

Le trait est dit non *dyn compatible*. C'est la raison pour laquelle `Clone` ne peut pas s'utiliser en `dyn Clone`. Tu le croiseras rarement dans tes propres traits ; retiens que le compilateur explique pourquoi, méthode par méthode.

**Venant du C :** l'objet trait est la struct de pointeurs de fonctions que tu passais avec un `void *` (`struct ops { int (*verifier)(void *); }` et son contexte), le pointeur gras rassemblant les deux. Rust construit la table pour toi, une fois par couple type-trait, et vérifie les signatures. Le générique, lui, est ce qu'aurait donné une macro C qui recopie la fonction pour chaque type, mais vérifiée par le compilateur.

### 7.6 Les traits de la bibliothèque standard à connaître

Une poignée de traits de la std reviennent dans tout programme Rust. Tu en dérives la plupart, tu en écris quelques-uns à la main. Voici une `Version` de logiciel qui en implémente sept :

```rust
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Version {
    majeur: u32,
    mineur: u32,
    correctif: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.majeur, self.mineur, self.correctif)
    }
}

impl From<(u32, u32, u32)> for Version {
    fn from((majeur, mineur, correctif): (u32, u32, u32)) -> Self {
        Version {
            majeur,
            mineur,
            correctif,
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(texte: &str) -> Result<Self, Self::Error> {
        let nombres: Vec<u32> = texte
            .trim_start_matches('v')
            .split('.')
            .map(|morceau| morceau.parse::<u32>())
            .collect::<Result<_, _>>()
            .map_err(|e| format!("{texte} : {e}"))?;
        match nombres[..] {
            [majeur, mineur, correctif] => Ok(Version {
                majeur,
                mineur,
                correctif,
            }),
            _ => Err(format!("{texte} : il faut trois nombres")),
        }
    }
}

fn main() {
    let mut versions = vec![
        Version::from((1, 10, 0)),
        (1, 2, 3).into(),
        Version::try_from("v1.9.12").unwrap(),
        Version::default(),
    ];
    versions.sort();
    let texte: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
    println!("{}", texte.join(" < "));
    println!("max : {}", versions.iter().max().unwrap());

    let deployees: BTreeSet<Version> = versions.iter().copied().collect();
    println!("{} versions distinctes", deployees.len());

    for essai in ["2.0.1", "2.0", "2.x.1"] {
        let v: Result<Version, _> = essai.try_into();
        println!("{essai:>6} -> {v:?}");
    }
}
```

```
$ cargo run
v0.0.0 < v1.2.3 < v1.9.12 < v1.10.0
max : v1.10.0
4 versions distinctes
 2.0.1 -> Ok(Version { majeur: 2, mineur: 0, correctif: 1 })
   2.0 -> Err("2.0 : il faut trois nombres")
 2.x.1 -> Err("2.x.1 : invalid digit found in string")
```

- `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default` : dérivés, comme au chapitre 6. `Default` donne `v0.0.0`.
- `PartialOrd` et `Ord` dérivés comparent **les champs dans l'ordre de déclaration** : d'abord `majeur`, puis `mineur`, puis `correctif`. C'est exactement l'ordre des versions, et c'est pour ça que `v1.9.12 < v1.10.0`, là où un tri de chaînes aurait mis `v1.10.0` avant `v1.9.12`. `Ord` permet `sort()`, `max()` et les clés de `BTreeMap` et `BTreeSet`.
- `Display` à la main, comme au chapitre 6 : c'est lui qui donne `to_string()`.
- `From<(u32, u32, u32)>` : une conversion **qui ne peut pas échouer**. L'implémenter donne **gratuitement** `Into` dans l'autre sens : `(1, 2, 3).into()` marche parce que le type attendu (`Version`, dans le `vec!`) est connu. Règle : implémente `From`, utilise `into()`.
- `TryFrom<&str>` : une conversion **qui peut échouer**. Elle rend un `Result`, et `type Error = String` dit quel type d'erreur (le chapitre 8 fera mieux qu'une `String`). Même cadeau : `try_into()` dans l'autre sens. Note le `collect::<Result<_, _>>()` du chapitre 5 et le `?`, qui sort au premier nombre illisible, et le motif de tranche `[majeur, mineur, correctif]`, qui vérifie qu'il y en a exactement trois.

D'autres traits servent moins à décrire tes types qu'à rendre tes fonctions accueillantes, ou à brancher tes types sur le langage :

```rust
use std::path::{Path, PathBuf};

fn existe<P: AsRef<Path>>(chemin: P) -> bool {
    chemin.as_ref().exists()
}

fn crier<S: AsRef<str>>(texte: S) -> String {
    texte.as_ref().to_uppercase()
}

struct Rebours {
    reste: u32,
}

impl Iterator for Rebours {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.reste == 0 {
            return None;
        }
        self.reste -= 1;
        Some(self.reste + 1)
    }
}

struct Verrou {
    nom: String,
}

impl Drop for Verrou {
    fn drop(&mut self) {
        println!("verrou {} libéré", self.nom);
    }
}

fn main() {
    println!(
        "{} {} {}",
        existe("/etc/hosts"),
        existe(String::from("/nulle/part")),
        existe(PathBuf::from("/tmp"))
    );
    println!("{} {}", crier("ok"), crier(String::from("ko")));

    let tout: Vec<u32> = Rebours { reste: 5 }.collect();
    println!("{tout:?}");
    let pairs: u32 = Rebours { reste: 10 }.filter(|n| n % 2 == 0).sum();
    println!("somme des pairs : {pairs}");
    for (i, n) in (Rebours { reste: 3 }).enumerate() {
        print!("{i}:{n} ");
    }
    println!();

    let _a = Verrou {
        nom: "sauvegarde".to_string(),
    };
    {
        let _b = Verrou {
            nom: "migration".to_string(),
        };
        println!("migration en cours");
    }
    println!("fin de main");
}
```

```
$ cargo run
true false true
OK KO
[5, 4, 3, 2, 1]
somme des pairs : 30
0:3 1:2 2:1 
migration en cours
verrou migration libéré
fin de main
verrou sauvegarde libéré
```

- `AsRef<Path>` : « tout ce qui peut se voir comme un chemin ». `existe` accepte une `&str`, une `String`, un `PathBuf`, sans que l'appelant convertisse. Toute la std l'utilise : c'est pourquoi `fs::read_to_string("x")` et `fs::read_to_string(&chemin)` marchent tous les deux. `AsRef<str>` fait pareil pour le texte.
- `Iterator` : une seule méthode obligatoire, `next`, et un **type associé**, `type Item = u32`, qui dit ce que l'itérateur produit. En échange, tu reçois les soixante-dix méthodes par défaut du trait : `collect`, `filter`, `sum`, `enumerate`, tout le chapitre 5. Écrire `next`, c'est brancher ton type sur les boucles `for` et sur toute la boîte à outils.
- `Drop` : le code qui tourne quand la valeur sort de la portée (chapitre 4). `migration` est libéré à la fin du bloc intérieur, `sauvegarde` à la fin de `main`, dans l'ordre inverse de création. C'est ainsi qu'un `MutexGuard` rend le verrou, qu'un fichier se ferme, qu'une transaction s'annule si on ne l'a pas validée. Tu en écriras rarement, tu en profiteras tout le temps.

**Venant de Python :** tu connais tous ces traits sous un autre nom. `Display` est `__str__`, `Debug` est `__repr__`, `PartialEq` est `__eq__`, `PartialOrd` est `__lt__` et ses frères (ou `functools.total_ordering`), `Hash` est `__hash__`, `Iterator` est `__iter__` plus `__next__` (avec `None` au lieu de `StopIteration`), `Drop` est `__exit__` d'un `with` qui serait toujours là, `Add` (section suivante) est `__add__`. La différence : le trait est déclaré, vérifié, et `derive` écrit les plus ennuyeux.

### 7.7 Surcharge d'opérateurs : `Add` et ses cousins

Les opérateurs de Rust sont des traits de `std::ops`. `a + b` est du sucre pour `Add::add(a, b)`, `a += b` pour `AddAssign::add_assign(&mut a, b)`. Implémente le trait, et l'opérateur marche sur ton type :

```rust
use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Octets(u64);

impl Add for Octets {
    type Output = Octets;

    fn add(self, autre: Octets) -> Octets {
        Octets(self.0 + autre.0)
    }
}

impl Add<u64> for Octets {
    type Output = Octets;

    fn add(self, autre: u64) -> Octets {
        Octets(self.0 + autre)
    }
}

impl AddAssign for Octets {
    fn add_assign(&mut self, autre: Octets) {
        self.0 += autre.0;
    }
}

impl std::iter::Sum for Octets {
    fn sum<I: Iterator<Item = Octets>>(iter: I) -> Octets {
        iter.fold(Octets(0), |acc, o| acc + o)
    }
}

impl fmt::Display for Octets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1} Mio", self.0 as f64 / (1024.0 * 1024.0))
    }
}

fn main() {
    let wal = Octets(16 * 1024 * 1024);
    let base = Octets(512 * 1024 * 1024);
    println!("{}", base + wal);
    println!("{}", base + 1024 * 1024);
    let mut total = Octets(0);
    total += wal;
    total += wal;
    println!("{total}");
    let fichiers = [wal, base, Octets(3 * 1024 * 1024)];
    let somme: Octets = fichiers.iter().copied().sum();
    println!("{somme} {}", somme > base);
}
```

```
$ cargo run
528.0 Mio
513.0 Mio
32.0 Mio
531.0 Mio true
```

- `impl Add for Octets` : `type Output = Octets` dit ce que rend l'addition. Le plus souvent `Self`, mais pas toujours (une date moins une date donne une durée).
- `impl Add<u64> for Octets` : une **deuxième** implémentation, pour ajouter un entier nu. `Add` a un paramètre de type, le type de droite, qui vaut `Self` par défaut. C'est ce que la section suivante appelle un trait générique.
- `AddAssign` pour `+=`, et `Sum` pour que `sum()` sache additionner des `Octets` : un `fold` de trois lignes, et tes types rejoignent les chaînes d'itérateurs.
- `PartialOrd` dérivé suffit pour `>` : la comparaison passe par la valeur intérieure.

Il y a un trait par opérateur : `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `Not`, `Index` pour `v[i]`. Sers-t'en quand l'opérateur a un sens évident pour le lecteur (additionner des tailles, des durées, des vecteurs). Un `+` qui ouvre une connexion n'a rien à faire là.

### 7.8 Traits génériques et types associés

Deux façons de laisser un « trou de type » dans un trait, et la différence compte :

- **Paramètre générique** : `trait Add<Rhs = Self>`, `trait From<T>`. Un même type peut implémenter le trait **plusieurs fois**, une par valeur du paramètre. `Octets` implémente `Add<Octets>` et `Add<u64>` ; `Version` pourrait implémenter `From<(u32, u32, u32)>` et `From<[u32; 3]>`. Le compilateur choisit l'implémentation selon le type de l'argument.
- **Type associé** : `type Item` dans `Iterator`, `type Output` dans `Add`, `type Error` dans `TryFrom`. Une seule valeur **par implémentation**. `Rebours` ne peut être un itérateur que d'une seule sorte d'éléments, et c'est bien : si `Iterator` avait un paramètre `Iterator<Item>`, il faudrait écrire `Iterator<u32>` à chaque usage et le compilateur ne saurait jamais ce que `collect` reçoit.

La règle pour tes propres traits : un type associé quand le type qui implémente **détermine** la réponse (un itérateur sait ce qu'il produit), un paramètre générique quand plusieurs réponses ont un sens pour le même type (on peut convertir **depuis** plusieurs types). Dans le doute, type associé : c'est plus simple pour l'appelant, qui écrit `impl Iterator<Item = u16>` sans rien deviner.

### 7.9 La règle de l'orphelin

Tu peux implémenter **ton** trait pour n'importe quel type (`impl Controle for u16`), et n'importe quel trait pour **ton** type (`impl Display for Version`). Ce que tu ne peux pas faire, c'est implémenter un trait étranger pour un type étranger :

```
$ cargo build
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
 --> src/main.rs:3:1
  |
3 | impl fmt::Display for Vec<u16> {
  | ^^^^^^^^^^^^^^^^^^^^^^--------
  |                       |
  |                       `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

La raison est la cohérence. Si ta crate et une autre implémentaient chacune `Display` pour `Vec<u16>`, laquelle gagnerait dans un programme qui utilise les deux ? Rust interdit le conflit à la source : pour chaque couple trait-type, une seule crate a le droit d'écrire l'implémentation, celle qui possède le trait ou le type. C'est la *règle de l'orphelin* (une implémentation doit avoir un parent dans ta crate).

La sortie est celle que suggère la dernière note du message : un **newtype**, la struct tuple du chapitre 6 qui enveloppe le type étranger. Il est à toi, tu lui implémentes ce que tu veux :

```rust
use std::fmt;

struct Ports(Vec<u16>);

impl fmt::Display for Ports {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let textes: Vec<String> = self.0.iter().map(|p| p.to_string()).collect();
        write!(f, "{} ports : {}", self.0.len(), textes.join("/"))
    }
}

trait Etiquette {
    fn etiquette(&self) -> String;
}

impl<T: fmt::Display> Etiquette for T {
    fn etiquette(&self) -> String {
        format!("<{self}>")
    }
}

#[derive(Debug)]
struct Hote {
    nom: String,
    derniere_vue: u64,
}

impl PartialEq for Hote {
    fn eq(&self, autre: &Hote) -> bool {
        self.nom == autre.nom
    }
}

fn main() {
    let p = Ports(vec![22, 80, 443]);
    println!("{p}");
    println!(
        "{} {} {}",
        42.etiquette(),
        "db-01".etiquette(),
        p.etiquette()
    );

    let a = Hote {
        nom: "db-01".to_string(),
        derniere_vue: 1_761_350_000,
    };
    let b = Hote {
        nom: "db-01".to_string(),
        derniere_vue: 1_761_353_600,
    };
    println!(
        "même hôte : {}, vus à {} et {}",
        a == b,
        a.derniere_vue,
        b.derniere_vue
    );
}
```

```
$ cargo run
3 ports : 22/80/443
<42> <db-01> <3 ports : 22/80/443>
même hôte : true, vus à 1761350000 et 1761353600
```

`Ports(Vec<u16>)` ne coûte rien en mémoire (même taille que le `Vec`), et il a son `Display`. Le reste de l'exemple montre les deux dernières notions du chapitre.

**`derive` ou à la main.** `derive(PartialEq)` compare **tous** les champs. Pour `Hote`, deux observations du même hôte à des heures différentes doivent être égales : on écrit `eq` à la main, sur le seul champ `nom`. Même logique pour `Ord` (trier par priorité puis par nom, quand les champs ne sont pas dans cet ordre), `Debug` (masquer un mot de passe), `Default` (un port par défaut à 5432 plutôt que 0). Règle : `derive` tant que le comportement champ par champ est le bon, à la main dès qu'il ne l'est plus. Et garde la cohérence : si deux valeurs sont égales pour `PartialEq`, elles doivent avoir le même `Hash`, sinon une `HashMap` se met à perdre des clés.

**L'implémentation générale** (*blanket impl*). `impl<T: Display> Etiquette for T` implémente `Etiquette` pour **tous** les types affichables d'un coup : `42`, `"db-01"` et `Ports` reçoivent `etiquette()` sans une ligne de plus. La std en est pleine : c'est `impl<T: Display> ToString for T` qui t'a donné `to_string()` au chapitre 6 dès que tu as écrit `Display`, et `impl<T, U: From<T>> Into<U> for T` qui donne `into()` à qui implémente `From`.

### 7.10 Pour le labo

Le [labo 07](../labs/07-formes-et-io/README.md) met tout ça en œuvre sur des formes géométriques. Un trait `Forme` avec une méthode par défaut `decrire`, trois types (`Cercle`, `Rectangle`, `Triangle` avec la formule de Héron), un `From<(f64, f64)>` pour construire un rectangle depuis un couple, `Display` pour les trois, et deux versions de la même fonction, `aire_totale<T: Forme>` en générique et `aire_totale_dyn` sur des `Box<dyn Forme>`, pour sentir la différence. Puis un trait `Sortie` implémenté par la console et par un tampon en mémoire, qui permet de tester un rapport sans rien capturer sur la sortie standard : c'est l'injection de dépendance de Rust, sans framework. Et un `Compteur` qui implémente `Iterator`, pour profiter de `filter` et `sum` sur ton propre type.

### À retenir

- Un trait déclare des méthodes ; `impl Trait for Type` les fournit, explicitement. Une méthode par défaut s'appuie sur les méthodes obligatoires.
- Un générique `<T: Borne>` marche pour tout type qui implémente la borne ; `where` les range quand il y en a plusieurs. L'erreur, s'il en manque une, est à l'appel et nomme le trait.
- `impl Trait` en argument est un générique abrégé ; en retour, c'est un type unique que tu ne nommes pas (itérateurs, fermetures).
- `dyn Trait` derrière `&` ou `Box` range des types différents ensemble ; l'appel passe par une table virtuelle, et le pointeur fait 16 octets.
- Générique par défaut (monomorphisation, aucun coût à l'exécution) ; `dyn` pour les collections hétérogènes et les choix faits à l'exécution. Ensemble fermé de cas : un enum.
- Les traits de la std à connaître : `Display`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`, `From` (qui donne `Into`), `TryFrom`, `AsRef`, `Iterator`, `Drop`.
- Les opérateurs sont des traits (`Add`, `AddAssign`, `Index`...) ; implémente-les quand le sens est évident.
- Type associé quand l'implémentation détermine le type (`Iterator::Item`), paramètre générique quand plusieurs implémentations ont un sens (`From<T>`).
- Règle de l'orphelin : pas de trait étranger sur un type étranger ; enveloppe-le dans un newtype.
- `derive` tant que la comparaison champ par champ est la bonne, à la main sinon ; une *blanket impl* `impl<T: Display> X for T` couvre tous les types d'un coup.

---

← [6. Structs, enums, méthodes, Option](06-structs-enums-option.md) · [Sommaire](../README.md) · [8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic](08-erreurs.md) →
