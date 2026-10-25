# 5. Collections et itérateurs

*Rust de zéro à la prod : chapitre 5 sur 16.* ← [4. Possession et emprunts : le cœur de Rust](04-possession-et-emprunts.md) · [Sommaire](../README.md) · [6. Structs, enums, méthodes, Option](06-structs-enums-option.md) →

Tu sais maintenant qui possède quoi, et ce qu'un `&` ou un `&mut` a le droit de faire. Ce chapitre met ces règles au travail sur les structures que tu utilises tous les jours : la liste qui grandit (`Vec`), le dictionnaire (`HashMap`, `BTreeMap`), l'ensemble (`HashSet`), la file (`VecDeque`) et la chaîne de caractères, qui n'est qu'un `Vec<u8>` avec une promesse.

La seconde moitié parle des **itérateurs**, et c'est elle qui compte. En Go, tu écris des boucles `for range` avec un `append` dedans, partout. En Rust, tu écris des chaînes `iter().filter().map().collect()`, et ce n'est pas une affaire de goût : c'est aussi rapide que la boucle (le compilateur produit la même chose), c'est plus court, et chaque étape dit ce qu'elle fait. Quand tu sauras lire et écrire ces chaînes, la moitié du code Rust que tu croises deviendra lisible.

### 5.1 `Vec<T>` : le tableau qui grandit

`Vec<T>` est un tableau d'éléments de type `T`, alloué sur le tas, qui grandit quand on ajoute. C'est la collection par défaut : si tu hésites, prends un `Vec`.

```rust
fn main() {
    let mut ports: Vec<u16> = Vec::new();
    ports.push(22);
    ports.push(443);
    ports.push(5432);
    let hotes = vec!["web-01", "db-01", "cache-01"];

    println!("{ports:?} longueur {}", ports.len());
    println!("premier hôte : {}", hotes[0]);
    println!("get(1) = {:?}, get(9) = {:?}", hotes.get(1), hotes.get(9));

    if let Some(dernier) = ports.last() {
        println!("dernier port : {dernier}");
    }
    let retire = ports.pop();
    println!("pop : {retire:?}, reste {ports:?}");

    println!("{}", hotes[9]);
}
```

```
$ cargo run
[22, 443, 5432] longueur 3
premier hôte : web-01
get(1) = Some("db-01"), get(9) = None
dernier port : 5432
pop : Some(5432), reste [22, 443]

thread 'main' (55101248) panicked at src/main.rs:18:25:
index out of bounds: the len is 3 but the index is 9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

- `Vec::new()` crée un vecteur vide, `vec![...]` un vecteur rempli. Le type des éléments est déduit (`Vec<&str>` pour `hotes`) ; pour `ports`, on l'a écrit, sinon le compilateur aurait choisi `i32`.
- `push` exige `let mut` : ajouter modifie le vecteur, et le chapitre 4 t'a appris qu'on ne modifie que ce qu'on a déclaré mutable.
- `hotes[0]` rend l'élément, et **panique** si l'index est hors bornes (dernière ligne). `hotes.get(9)` rend un `Option<&T>` : `Some(&élément)` ou `None`. Le chapitre 6 détaille `Option` ; pour l'instant, retiens que `get` ne plante jamais.
- `last()`, `first()` et `pop()` rendent aussi des `Option`, parce qu'un vecteur vide n'a ni premier ni dernier élément.

Quand utiliser l'un ou l'autre ? `v[i]` quand un index hors bornes est un bug (tu viens de vérifier la longueur, tu parcours `0..v.len()`) : le panic te le dit tout de suite, à la bonne ligne. `v.get(i)` quand l'index vient de dehors (un argument, une ligne de fichier, une requête) : c'est un cas normal, pas un bug, et tu le traites.

**Venant de Go :** `Vec<T>` est la slice `[]T`, et `push` est `append`. Deux différences. `append` renvoie une nouvelle slice que tu dois réaffecter (`s = append(s, x)`), et si tu oublies, ou si deux slices partagent le même tableau, tu as des surprises ; `push` modifie le vecteur en place, et le vérificateur d'emprunts interdit qu'une autre référence le regarde pendant ce temps. Et l'index hors bornes panique dans les deux langages, mais Rust t'offre `get` pour ne pas paniquer, là où Go te laisse écrire le test `if i < len(s)` à la main.

Sous le capot, un `Vec` est trois mots sur la pile : un pointeur vers le tas, une longueur et une **capacité**, le nombre d'éléments que la zone allouée peut contenir. Quand la longueur atteint la capacité, `push` alloue une zone plus grande, copie, et libère l'ancienne :

```rust
fn main() {
    let mut v: Vec<u32> = Vec::new();
    let mut cap = v.capacity();
    println!("vide : len {} cap {}", v.len(), cap);
    for i in 0..20 {
        v.push(i);
        if v.capacity() != cap {
            cap = v.capacity();
            println!("après {:>2} push : cap {}", v.len(), cap);
        }
    }

    let mut lignes = Vec::with_capacity(1000);
    lignes.extend(["a", "b"]);
    let autres = vec!["c", "d"];
    lignes.extend(autres);
    println!("{lignes:?} cap {}", lignes.capacity());
}
```

```
$ cargo run
vide : len 0 cap 0
après  1 push : cap 4
après  5 push : cap 8
après  9 push : cap 16
après 17 push : cap 32
["a", "b", "c", "d"] cap 1000
```

La capacité double à chaque fois (après un départ à 4 pour des petits éléments), donc `push` coûte en moyenne un temps constant. Si tu connais la taille à l'avance, `Vec::with_capacity(n)` réserve d'un coup et évite les recopies. `extend` ajoute tout ce qu'on lui donne : un tableau, un autre vecteur, n'importe quel itérateur.

**Venant du C :** c'est le `realloc` que tu écrivais à la main, avec la longueur et la capacité rangées à côté du pointeur, et la libération faite par `Drop` à la fin de la portée (chapitre 4). Pas d'oubli de `free`, pas de pointeur qui traîne vers l'ancienne zone après une réallocation : le vérificateur d'emprunts refuse de compiler un `&v[0]` gardé pendant un `v.push(...)`.

### 5.2 Trier, dédoublonner, filtrer sur place

Les opérations de tous les jours sont des méthodes du vecteur, et elles modifient sur place :

```rust
fn main() {
    let mut ports = vec![5432, 22, 443, 22, 80, 443, 443];
    ports.sort();
    println!("trié     : {ports:?}");
    ports.dedup();
    println!("dédoublé : {ports:?}");
    ports.retain(|&p| p != 80);
    println!("sans 80  : {ports:?}");

    let mut hotes: Vec<(&str, f64)> = vec![
        ("web-01", 0.7),
        ("db-01", 0.2),
        ("cache-01", 0.5),
        ("db-02", 0.2),
    ];
    hotes.sort_by_key(|&(nom, _)| nom.len());
    println!("par longueur de nom : {hotes:?}");
    hotes.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("charge décroissante : {hotes:?}");

    let mut charges = vec![0.7, f64::NAN, 0.2, 0.5];
    charges.sort_by(f64::total_cmp);
    println!("{charges:?}");
}
```

```
$ cargo run
trié     : [22, 22, 80, 443, 443, 443, 5432]
dédoublé : [22, 80, 443, 5432]
sans 80  : [22, 443, 5432]
par longueur de nom : [("db-01", 0.2), ("db-02", 0.2), ("web-01", 0.7), ("cache-01", 0.5)]
charge décroissante : [("web-01", 0.7), ("cache-01", 0.5), ("db-01", 0.2), ("db-02", 0.2)]
[0.2, 0.5, 0.7, NaN]
```

- `sort()` trie dans l'ordre naturel. `dedup()` retire les doublons **consécutifs** : il faut trier avant, sinon `[22, 443, 22]` reste tel quel.
- `retain(|&p| p != 80)` garde les éléments pour lesquels la fermeture renvoie `true`. La syntaxe `|x| ...` est une **fermeture** (section 5.10), le `lambda` de Python. Le `&p` dans les paramètres déstructure la référence que `retain` passe : `p` est un entier, pas un `&i32`.
- `sort_by_key(|&(nom, _)| nom.len())` trie selon une clé calculée. `sort_by(|a, b| ...)` trie avec une fonction de comparaison qui renvoie un `Ordering` (`Less`, `Equal`, `Greater`) ; inverser `a` et `b` donne l'ordre décroissant.
- Les deux sont **stables** : à clé égale, l'ordre d'origine est conservé. Regarde `db-01` et `db-02`, même longueur de nom, même charge : ils restent dans l'ordre où ils étaient. Le labo 05 en a besoin, et le teste.
- `sort_unstable()`, `sort_unstable_by_key()` : même résultat sur les clés, sans garantie d'ordre pour les égaux, un peu plus rapides et sans allocation. Prends-les quand les éléments égaux sont indiscernables, des entiers par exemple.

Et les flottants ? Essaie de trier un `Vec<f64>` avec `sort()` :

```
$ cargo build
error[E0277]: the trait bound `{float}: Ord` is not satisfied
 --> src/main.rs:3:13
  |
3 |     charges.sort();
  |             ^^^^ the trait `Ord` is not implemented for `{float}`
```

`sort` exige que le type soit totalement ordonné (le trait `Ord`, chapitre 7). Or un `f64` peut valoir `NaN`, qui n'est ni plus petit, ni plus grand, ni égal à quoi que ce soit, pas même à lui-même. Rust refuse de trier sans que tu dises quoi faire de `NaN`. La réponse standard est `f64::total_cmp`, qui définit un ordre total (les `NaN` à la fin, `-0.0` avant `0.0`) : `charges.sort_by(f64::total_cmp)`, comme dans la dernière ligne de l'exemple. Tu passes la fonction elle-même, sans fermeture autour.

**Venant de Go :** `sort.Float64s` met les `NaN` au début sans te demander ton avis, et `slices.SortFunc` avec `cmp.Compare` fait pareil. Rust t'oblige à choisir, une fois, explicitement.

### 5.3 Les tranches : `&[T]`

Une tranche (*slice*) est une vue sur une suite d'éléments contigus, sans en être propriétaire : un pointeur et une longueur. `&[T]` en lecture, `&mut [T]` en écriture. C'est ce que tu dois **prendre en paramètre** quand une fonction lit une suite d'éléments, parce qu'un `&Vec<T>`, un tableau `[T; N]` et un morceau de vecteur se convertissent tous en `&[T]` :

```rust
fn moyenne(valeurs: &[f64]) -> f64 {
    valeurs.iter().sum::<f64>() / valeurs.len() as f64
}

fn main() {
    let latences = vec![12.0, 15.0, 11.0, 80.0, 14.0, 13.0];
    let tableau = [1.0, 2.0, 3.0];

    println!("tout      : {:.1}", moyenne(&latences));
    println!("3 premiers: {:.1}", moyenne(&latences[..3]));
    println!("tableau   : {:.1}", moyenne(&tableau));

    for paquet in latences.chunks(4) {
        println!("paquet {paquet:?}");
    }
    for w in latences.windows(2) {
        if w[1] > w[0] * 2.0 {
            println!("saut de {} à {}", w[0], w[1]);
        }
    }
    let (debut, fin) = latences.split_at(2);
    println!("{debut:?} | {fin:?}");
    println!("contient 80 : {}", latences.contains(&80.0));
}
```

```
$ cargo run
tout      : 24.2
3 premiers: 12.7
tableau   : 2.0
paquet [12.0, 15.0, 11.0, 80.0]
paquet [14.0, 13.0]
saut de 11 à 80
[12.0, 15.0] | [11.0, 80.0, 14.0, 13.0]
contient 80 : true
```

- `moyenne(&latences)` : le `&Vec<f64>` devient un `&[f64]` tout seul (une *deref coercion*, le chapitre 10 l'explique). `&latences[..3]` prend les trois premiers, `&latences[2..]` tout à partir du troisième, comme en Python, sans copie.
- `chunks(4)` découpe en paquets de 4 (le dernier est plus court) : c'est ce qu'il te faut pour envoyer des lignes à une base par lots de 1000. `windows(2)` fait glisser une fenêtre de 2 : chaque élément avec son voisin, pour détecter un saut. Les deux rendent des tranches, pas des copies.
- `split_at`, `contains`, `iter`, `first`, `binary_search` : presque toutes les méthodes que tu crois appeler sur un `Vec` sont en réalité des méthodes de la tranche. C'est pour ça qu'elles marchent aussi sur les tableaux.

**Piège :** écrire `fn moyenne(valeurs: &Vec<f64>)` compile, mais clippy te dira `writing &Vec instead of &[_] involves a new object where a slice will do`. Avec `&Vec`, la fonction refuse un tableau ou un morceau de vecteur. Prends `&[T]`, et `&str` plutôt que `&String`, pour la même raison.

### 5.4 `HashMap` et `BTreeMap` : l'API `entry`

`HashMap<K, V>` est le dictionnaire, avec un accès en temps constant et **aucun ordre** d'itération. `BTreeMap<K, V>` est un arbre trié : accès en temps logarithmique, itération **dans l'ordre des clés**. Les deux s'importent depuis `std::collections` :

```rust
use std::collections::{BTreeMap, HashMap};

fn main() {
    let log = "web-01 db-01 web-01 cache-01 web-01 db-01";

    let mut compte: HashMap<&str, usize> = HashMap::new();
    for hote in log.split_whitespace() {
        *compte.entry(hote).or_insert(0) += 1;
    }
    println!("{compte:?}");
    println!(
        "web-01 : {:?}, absent : {:?}",
        compte.get("web-01"),
        compte.get("x")
    );

    let mut trie: BTreeMap<&str, usize> = BTreeMap::new();
    for hote in log.split_whitespace() {
        *trie.entry(hote).or_default() += 1;
    }
    println!("{trie:?}");

    let mut par_role: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for hote in ["web-01", "db-01", "web-02"] {
        let role = hote.split('-').next().unwrap();
        par_role.entry(role).or_default().push(hote);
    }
    for (role, hotes) in &par_role {
        println!("{role} -> {hotes:?}");
    }

    let ancien = compte.insert("db-01", 10);
    println!("insert a remplacé {ancien:?}");
    compte.remove("cache-01");
    println!("contient cache-01 : {}", compte.contains_key("cache-01"));
}
```

```
$ cargo run
{"cache-01": 1, "db-01": 2, "web-01": 3}
web-01 : Some(3), absent : None
{"cache-01": 1, "db-01": 2, "web-01": 3}
db -> ["db-01"]
web -> ["web-01", "web-02"]
insert a remplacé Some(2)
contient cache-01 : false
```

La première ligne de la sortie change d'un lancement à l'autre : j'ai lancé le programme trois fois, voici les trois premières lignes :

```
{"cache-01": 1, "web-01": 3, "db-01": 2}
{"web-01": 3, "db-01": 2, "cache-01": 1}
{"web-01": 3, "cache-01": 1, "db-01": 2}
```

`HashMap` utilise une fonction de hachage avec une graine aléatoire, différente à chaque lancement, pour résister aux attaques par collisions (un client qui t'envoie des clés choisies pour tout ranger dans la même case). L'ordre est donc imprévisible. Si tu affiches, si tu compares dans un test, si tu écris un fichier : `BTreeMap`, ou tu tries avant.

L'outil qui change tout est **`entry`**. `compte.entry(hote)` cherche la clé une seule fois et rend une « entrée », occupée ou vacante. `.or_insert(0)` y met 0 si elle était vide, et rend dans les deux cas un `&mut usize` vers la valeur ; le `*` devant déréférence pour ajouter 1. `.or_default()` fait pareil avec la valeur par défaut du type : 0 pour un entier, chaîne vide, vecteur vide. Le motif `entry(cle).or_default().push(x)` est la façon de grouper : une ligne, une recherche, aucun `if`.

Le reste de l'API est sans surprise : `get` rend un `Option<&V>`, `insert` rend l'ancienne valeur s'il y en avait une, `remove`, `contains_key`, `len`. `for (cle, valeur) in &map` parcourt les paires par référence.

**Venant de Python :** `entry(k).or_insert(v)` est `dict.setdefault(k, v)`, et `or_default()` est `collections.defaultdict`, sans avoir à choisir le type de dictionnaire à la création. `*compte.entry(mot).or_insert(0) += 1` est le `compte[mot] = compte.get(mot, 0) + 1` que tu écris depuis toujours, avec une seule recherche au lieu de deux. Et `compte["absent"]` panique en Rust comme il lève `KeyError` en Python : préfère `get`.

**Venant de Go :** la `map` de Go rend la valeur zéro pour une clé absente, ce qui rend `m[k]++` possible directement, et l'ordre d'itération est volontairement aléatoire, comme ici. Rust n'a pas de valeur zéro implicite : `get` rend `None`, et `entry` remplace l'idiome `v, ok := m[k]; if !ok { ... }`. Go n'a pas de map triée dans la bibliothèque standard ; Rust a `BTreeMap`.

### 5.5 `HashSet`, `BTreeSet`, `VecDeque`

Un ensemble est une map sans valeurs : il dit si un élément est présent, et c'est tout. `VecDeque` est une file à deux bouts, où ajouter et retirer aux deux extrémités coûte un temps constant.

```rust
use std::collections::{BTreeSet, HashSet, VecDeque};

fn main() {
    let prevus: HashSet<&str> = ["web-01", "web-02", "db-01"].into_iter().collect();
    let vus: HashSet<&str> = ["web-01", "db-01", "inconnu"].into_iter().collect();

    let mut manquants: Vec<_> = prevus.difference(&vus).collect();
    manquants.sort();
    println!("manquants : {manquants:?}");
    let intrus: BTreeSet<_> = vus.difference(&prevus).collect();
    println!("intrus    : {intrus:?}");
    println!("communs   : {}", prevus.intersection(&vus).count());

    let mut deja = HashSet::new();
    for ip in ["10.0.0.1", "10.0.0.2", "10.0.0.1"] {
        if !deja.insert(ip) {
            println!("doublon : {ip}");
        }
    }

    let mut file: VecDeque<u32> = VecDeque::new();
    for evenement in 1..=5 {
        file.push_back(evenement);
        if file.len() > 3 {
            file.pop_front();
        }
    }
    println!("trois derniers : {file:?}");
    file.push_front(0);
    println!("{:?} {:?}", file.front(), file.back());
}
```

```
$ cargo run
manquants : ["web-02"]
intrus    : {"inconnu"}
communs   : 2
doublon : 10.0.0.1
trois derniers : [3, 4, 5]
Some(0) Some(5)
```

- `difference`, `intersection`, `union` font les opérations d'ensembles. Ici : quels hôtes de l'inventaire n'ont pas répondu, lesquels ont répondu sans y être. `BTreeSet` est l'ensemble trié, pour un affichage stable.
- `insert` rend `false` si l'élément y était déjà : c'est le test de doublon en une ligne.
- `VecDeque` avec `push_back` et `pop_front` est une file FIFO ; bornée à trois éléments, c'est un tampon circulaire des derniers événements. Un `Vec` ferait l'affaire pour une pile (`push` et `pop` au même bout), mais retirer au début d'un `Vec` décale tout le reste.

**Venant de Go :** pas de type ensemble en Go, on écrit `map[string]struct{}` ; pas de file à deux bouts non plus, on bricole avec une slice ou `container/list`. Ici, tout est dans `std::collections`.

### 5.6 `String` : un `Vec<u8>` qui promet l'UTF-8

Le chapitre 2 t'a présenté `String` et `&str`. Vu d'ici, c'est simple : une `String` est un `Vec<u8>` dont le contenu est garanti être de l'UTF-8 valide, et `&str` est la tranche correspondante. Toutes les manipulations de texte d'un outil d'administration s'écrivent avec une poignée de méthodes :

```rust
fn main() {
    let mot = "Liège";
    println!("{} octets, {} caractères", mot.len(), mot.chars().count());
    println!("octets : {:?}", mot.bytes().collect::<Vec<u8>>());
    println!("à l'envers : {}", mot.chars().rev().collect::<String>());

    let mut s = String::with_capacity(32);
    s.push_str("db-01");
    s.push(':');
    s += "5432";
    println!("{s}");

    let conf = "  port = 5432 \n# commentaire\n\nhote=db-01\n";
    for ligne in conf.lines() {
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }
        if let Some((cle, valeur)) = ligne.split_once('=') {
            println!("[{}] = [{}]", cle.trim(), valeur.trim());
        }
    }

    let champs: Vec<&str> = "a,b,,c".split(',').collect();
    println!("{champs:?}");
    let mots: Vec<&str> = "  un   deux\ttrois ".split_whitespace().collect();
    println!("{mots:?}");

    let port: u16 = "5432".parse().unwrap();
    let rate = "70000".parse::<u16>();
    let pas_un_nombre = "pgsql".parse::<u16>();
    println!("{port} {rate:?} {pas_un_nombre:?}");
}
```

```
$ cargo run
6 octets, 5 caractères
octets : [76, 105, 195, 168, 103, 101]
à l'envers : egèiL
db-01:5432
[port] = [5432]
[hote] = [db-01]
["a", "b", "", "c"]
["un", "deux", "trois"]
5432 Err(ParseIntError { kind: PosOverflow }) Err(ParseIntError { kind: InvalidDigit })
```

- `len()` compte les **octets**. `chars()` itère sur les caractères Unicode, `bytes()` sur les octets : `è` en prend deux, 195 et 168. Inverser une chaîne se fait sur les `chars`, jamais sur les octets.
- `push_str` ajoute une `&str`, `push` un `char`, `+=` est un raccourci de `push_str`.
- `lines()` découpe en lignes (et gère `\r\n`), `trim()` retire les blancs aux deux bouts, `split_once('=')` coupe au premier `=` et rend un `Option<(&str, &str)>`. C'est le parseur de fichier de configuration du labo 08 en trois lignes.
- `split(',')` garde les champs vides (le `""` entre les deux virgules), `split_whitespace()` découpe sur n'importe quelle suite de blancs et ne rend jamais de champ vide.
- `parse::<u16>()` convertit, et rend un `Result` : `Ok(5432)`, ou une erreur qui dit pourquoi (`PosOverflow` pour 70000, qui ne tient pas dans un `u16`, `InvalidDigit` pour `pgsql`). Le chapitre 8 est consacré à ces erreurs.

Toutes ces méthodes rendent des `&str` qui **empruntent** la chaîne d'origine : aucune copie, aucune allocation. Tu ne paies une `String` neuve que quand tu en as besoin (`to_string()`, `to_lowercase()`, `format!`).

**Piège :** l'index d'une chaîne est un index d'**octets**, et couper au milieu d'un caractère panique :

```rust
fn main() {
    let mot = String::from("Liège");
    println!("{}", &mot[0..4]);
    println!("{}", &mot[0..3]);
}
```

```
$ cargo run
Liè

thread 'main' (55101417) panicked at src/main.rs:4:24:
byte index 3 is not a char boundary; it is inside 'è' (bytes 2..4) of `Liège`
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Le message est précis : l'octet 3 est au milieu du `è`, qui occupe les octets 2 et 3. Pour les textes venus de l'extérieur, préfère `chars()`, `split`, `find` (qui rendent des positions valides) aux index en dur.

**Venant de Go :** même modèle exactement : une `string` Go est une suite d'octets, `len` compte les octets, `range` sur une chaîne donne les runes comme `chars()`. La différence : Go accepte une `string` qui contient de l'UTF-8 invalide, Rust non (pour des octets quelconques, c'est `Vec<u8>`, et pour un nom de fichier, `OsString`, chapitre 12).

### 5.7 Trois façons d'itérer : `iter`, `iter_mut`, `into_iter`

Une collection offre trois itérateurs, et ils correspondent exactement aux trois façons d'accéder à une valeur du chapitre 4 :

| Appel | Donne | La collection après |
|---|---|---|
| `v.iter()` | des `&T` | intacte, on n'a fait que lire |
| `v.iter_mut()` | des `&mut T` | modifiée sur place |
| `v.into_iter()` | des `T` | consommée, déplacée, inutilisable |

Et `for x in &v` est un raccourci de `for x in v.iter()`, `for x in &mut v` de `iter_mut()`, `for x in v` de `into_iter()`.

```rust
fn main() {
    let mut hotes = vec![String::from("web-01"), String::from("db-01")];

    for h in hotes.iter() {
        println!("lu : {h}");
    }
    for h in hotes.iter_mut() {
        h.push_str(".prod");
    }
    println!("{hotes:?}");
    for h in hotes.into_iter() {
        let possede: String = h;
        println!("pris : {possede}");
    }
}
```

```
$ cargo run
lu : web-01
lu : db-01
["web-01.prod", "db-01.prod"]
pris : web-01.prod
pris : db-01.prod
```

Ajoute un `println!("{}", hotes.len());` à la fin, et le compilateur te rappelle ce que veut dire « consommé » :

```
$ cargo build
error[E0382]: borrow of moved value: `hotes`
  --> src/main.rs:15:20
   |
 2 |     let mut hotes = vec![String::from("web-01"), String::from("db-01")];
   |         --------- move occurs because `hotes` has type `Vec<String>`, which does not implement the `Copy` trait
...
11 |     for h in hotes.into_iter() {
   |                    ----------- `hotes` moved due to this method call
...
15 |     println!("{}", hotes.len());
   |                    ^^^^^ value borrowed here after move
   |
note: `into_iter` takes ownership of the receiver `self`, which moves `hotes`
```

C'est l'erreur E0382 du chapitre 4, appliquée à une boucle : `into_iter` prend le vecteur par valeur, chaque `String` sort du vecteur et appartient au corps de la boucle. Utilise `into_iter` quand tu veux **garder les éléments ailleurs** sans les copier (les ranger dans une map, les envoyer à un thread) ; `iter` dans tous les autres cas.

**Venant de Go :** `for i, v := range s` copie chaque élément dans `v`. Modifier `v` ne change pas la slice, et c'est un piège classique (il faut écrire `s[i].Champ = ...`). En Rust, le choix est dans le code : `iter()` interdit de modifier, `iter_mut()` donne une vraie référence vers l'élément, et `into_iter()` te donne l'élément lui-même.

### 5.8 Les adaptateurs : transformer sans boucle

Un itérateur est une valeur qui a une méthode `next()`, laquelle rend `Some(élément)` puis `None` à la fin. Les **adaptateurs** prennent un itérateur et en rendent un autre :

```rust
fn main() {
    let latences = [12, 15, 11, 80, 14, 13, 95];

    let lentes: Vec<i32> = latences.iter().filter(|&&ms| ms > 50).copied().collect();
    println!("lentes       : {lentes:?}");

    let doubles: Vec<i32> = latences.iter().map(|ms| ms * 2).take(3).collect();
    println!("map + take   : {doubles:?}");

    for (i, ms) in latences.iter().enumerate().skip(5) {
        println!("mesure {i} : {ms} ms");
    }

    let hotes = ["web-01", "db-01", "cache-01"];
    let ports = [443, 5432, 6379];
    let cibles: Vec<String> = hotes
        .iter()
        .zip(ports.iter())
        .map(|(h, p)| format!("{h}:{p}"))
        .collect();
    println!("zip          : {cibles:?}");

    let tout: Vec<&str> = hotes
        .iter()
        .chain(["dns-01"].iter())
        .rev()
        .copied()
        .collect();
    println!("chain + rev  : {tout:?}");

    let entrees = ["22", "http", "443", "", "5432"];
    let valides: Vec<u16> = entrees.iter().filter_map(|s| s.parse().ok()).collect();
    println!("filter_map   : {valides:?}");

    let lignes = ["a b", "c", "d e f"];
    let mots: Vec<&str> = lignes.iter().flat_map(|l| l.split(' ')).collect();
    println!("flat_map     : {mots:?}");

    let debut: Vec<_> = latences.iter().take_while(|&&ms| ms < 50).collect();
    println!("take_while   : {debut:?}");

    let mut it = [1, 1, 2, 3, 3, 3].into_iter().peekable();
    while let Some(x) = it.next() {
        let mut n = 1;
        while it.peek() == Some(&x) {
            it.next();
            n += 1;
        }
        print!("{x}x{n} ");
    }
    println!();
}
```

```
$ cargo run
lentes       : [80, 95]
map + take   : [24, 30, 22]
mesure 5 : 13 ms
mesure 6 : 95 ms
zip          : ["web-01:443", "db-01:5432", "cache-01:6379"]
chain + rev  : ["dns-01", "cache-01", "db-01", "web-01"]
filter_map   : [22, 443, 5432]
flat_map     : ["a", "b", "c", "d", "e", "f"]
take_while   : [12, 15, 11]
1x2 2x1 3x3 
```

- `filter(f)` garde ce pour quoi `f` rend `true`, `map(f)` transforme chaque élément. `copied()` transforme des `&i32` en `i32` (pour les types `Copy` ; `cloned()` pour les autres).
- `take(n)` s'arrête après n éléments, `skip(n)` saute les n premiers, `rev()` part de la fin, `enumerate()` ajoute l'index, `zip` marche sur deux itérateurs en même temps et s'arrête au plus court, `chain` les met bout à bout.
- `filter_map(f)` est `map` suivi d'un filtre sur les `None` : avec `s.parse().ok()`, il garde les entrées qui se convertissent et jette les autres en silence. C'est l'adaptateur le plus utile pour lire des données sales.
- `flat_map(f)` aplatit : chaque ligne donne plusieurs mots, et le résultat est la suite de tous les mots.
- `take_while(f)` prend tant que la condition tient, et s'arrête au premier échec (le 80), même si la suite contient d'autres petites valeurs.
- `peekable()` ajoute `peek()`, qui regarde l'élément suivant sans le consommer : de quoi écrire un compteur de répétitions (`1x2 2x1 3x3`) ou un petit analyseur.

**Piège :** les doubles `&` dans `filter(|&&ms| ms > 50)`. `latences.iter()` donne des `&i32`, et `filter` passe à sa fermeture une référence vers l'élément, donc un `&&i32`. Le motif `&&ms` déstructure les deux. Tu peux aussi écrire `filter(|ms| **ms > 50)`, ou mettre `copied()` avant le `filter` : alors c'est un simple `|&ms|`. Si tu te trompes, le compilateur te dit `error[E0308]: mismatched types` avec `expected &&_, found integer` : c'est presque toujours ça.

### 5.9 Les consommateurs : `collect` et les autres

Un adaptateur ne fait rien tout seul. Ce sont les **consommateurs** qui tirent les éléments un par un et produisent un résultat : une collection, un nombre, un booléen, un élément.

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Sonde {
    hote: &'static str,
    ms: u32,
    ok: bool,
}

fn main() {
    let sondes = [
        ("web-01", 12, true),
        ("db-01", 80, true),
        ("cache-01", 0, false),
        ("web-02", 15, true),
    ]
    .map(|(hote, ms, ok)| Sonde { hote, ms, ok });

    let total: u32 = sondes.iter().map(|s| s.ms).sum();
    let ok = sondes.iter().filter(|s| s.ok).count();
    println!("total {total} ms, {ok} OK sur {}", sondes.len());

    let plus_lente = sondes.iter().max_by_key(|s| s.ms);
    println!("plus lente : {:?}", plus_lente.map(|s| s.hote));
    let plus_rapide = sondes.iter().filter(|s| s.ok).min_by_key(|s| s.ms).unwrap();
    println!("plus rapide : {}", plus_rapide.hote);

    println!("une en panne : {}", sondes.iter().any(|s| !s.ok));
    println!("toutes < 100 ms : {}", sondes.iter().all(|s| s.ms < 100));
    println!(
        "position de db-01 : {:?}",
        sondes.iter().position(|s| s.hote == "db-01")
    );
    println!(
        "premier web : {:?}",
        sondes.iter().find(|s| s.hote.starts_with("web"))
    );
    println!("dernier : {:?}", sondes.iter().last().map(|s| s.hote));

    let resume = sondes.iter().fold(String::new(), |mut acc, s| {
        acc.push(if s.ok { '.' } else { 'X' });
        acc
    });
    println!("résumé : {resume}");

    let index: HashMap<&str, u32> = sondes.iter().map(|s| (s.hote, s.ms)).collect();
    println!("db-01 -> {:?}", index.get("db-01"));

    let noms = sondes.iter().map(|s| s.hote).collect::<Vec<_>>().join(", ");
    println!("{noms}");

    let initiales: String = sondes
        .iter()
        .map(|s| s.hote.chars().next().unwrap())
        .collect();
    println!("{initiales}");

    let tous: Result<Vec<u16>, _> = ["22", "443"].iter().map(|s| s.parse::<u16>()).collect();
    let un_faux: Result<Vec<u16>, _> = ["22", "ssh"].iter().map(|s| s.parse::<u16>()).collect();
    println!("{tous:?} {un_faux:?}");

    let charges = [0.7, 0.2, 0.5];
    let max = charges.iter().copied().max_by(f64::total_cmp);
    println!("charge max : {max:?}");
}
```

```
$ cargo run
total 107 ms, 3 OK sur 4
plus lente : Some("db-01")
plus rapide : web-01
une en panne : true
toutes < 100 ms : true
position de db-01 : Some(1)
premier web : Some(Sonde { hote: "web-01", ms: 12, ok: true })
dernier : Some("web-02")
résumé : ..X.
db-01 -> Some(80)
web-01, db-01, cache-01, web-02
wdcw
Ok([22, 443]) Err(ParseIntError { kind: InvalidDigit })
charge max : Some(0.7)
```

La `struct` en haut est une avance sur le chapitre 6 : lis-la comme une `struct` C ou Go, trois champs nommés. Ensuite :

- `sum()` et `count()` font ce qu'ils disent. `sum` a besoin de connaître le type du résultat : d'où le `let total: u32`.
- `max_by_key` et `min_by_key` rendent l'élément dont la clé est maximale ou minimale, pas la clé : un `Option<&Sonde>`, `None` si l'itérateur est vide. `max_by` prend une fonction de comparaison, d'où `max_by(f64::total_cmp)` pour des flottants, qui n'ont pas de `max` direct (même raison que pour `sort`).
- `any` et `all` s'arrêtent dès qu'ils connaissent la réponse. `position` rend l'index du premier élément qui convient, `find` l'élément lui-même, `last` le dernier.
- `fold(initial, f)` est la réduction générale : un accumulateur, une fonction qui le met à jour à chaque élément. Ici, un caractère par sonde.

Et **`collect`**, le plus important. Il rassemble les éléments dans n'importe quelle collection qui sait se construire depuis un itérateur (le trait `FromIterator`) : un `Vec`, une `HashMap` à partir de paires `(clé, valeur)`, une `String` à partir de `char` ou de `&str`, un `HashSet`. Il faut lui dire laquelle, soit par le type de la variable (`let index: HashMap<&str, u32> = ...collect()`), soit avec la syntaxe *turbofish* `collect::<Vec<_>>()`, où `_` laisse le compilateur deviner le type des éléments. Sans rien, il proteste :

```
$ cargo build
error[E0283]: type annotations needed
 --> src/main.rs:2:9
  |
2 |     let ports = ["22", "443"].iter().map(|s| s.parse::<u16>().unwrap()).collect();
  |         ^^^^^                                                           ------- type must be known at this point
  |
  = note: cannot satisfy `_: FromIterator<u16>`
help: consider giving `ports` an explicit type
  |
2 |     let ports: Vec<_> = ["22", "443"].iter().map(|s| s.parse::<u16>().unwrap()).collect();
  |              ++++++++
```

La dernière ligne de l'exemple montre la plus belle astuce de `collect` : un itérateur de `Result<T, E>` se collecte en `Result<Vec<T>, E>`. Si tout est `Ok`, tu as le vecteur ; à la première erreur, tu as l'erreur, et le reste n'est même pas parcouru. Valider une liste d'entrées tient en une ligne. Ça marche aussi avec `Option`.

**Venant de Python :** une chaîne d'itérateurs est une compréhension de liste écrite de gauche à droite. `[h.upper() for h in hotes if h.startswith("web")]` devient `hotes.iter().filter(|h| h.starts_with("web")).map(|h| h.to_uppercase()).collect::<Vec<_>>()`. Plus long sur une ligne, mais ça s'enchaîne sans limite (essaie d'imbriquer trois compréhensions), et chaque étape a un nom. `sum`, `any`, `all`, `min(key=...)`, `enumerate`, `zip`, `itertools.chain` : tu les connais déjà. `collect` est ce qui remplace le choix entre `[...]`, `{...}` et `"".join(...)`.

**Venant de Go :** Go n'a pas de `map` ni de `filter` sur les slices ; tu écris la boucle, et `slices` (Go 1.21) n'a ajouté que des fonctions ponctuelles (`Contains`, `Index`, `Max`). Les itérateurs Go 1.23 (`iter.Seq`) existent, mais sans adaptateurs dans la bibliothèque standard. En Rust, la chaîne est la façon normale d'écrire, et elle ne coûte rien : le compilateur la transforme en la boucle que tu aurais écrite, souvent vectorisée.

### 5.10 La paresse : rien ne se passe sans consommateur

Les adaptateurs sont **paresseux**. `map` ne fait aucun calcul quand tu l'appelles : il fabrique un itérateur qui calculera quand on lui demandera le suivant.

```rust
fn main() {
    let hotes = ["web-01", "db-01", "cache-01"];

    hotes.iter().map(|h| println!("sonde {h}"));

    let mut it = hotes.iter().map(|h| {
        println!("  calcul pour {h}");
        h.len()
    });
    println!("itérateur construit, rien calculé");
    let premier_long = it.find(|&n| n > 5);
    println!("{premier_long:?}");
}
```

```
$ cargo run
itérateur construit, rien calculé
  calcul pour web-01
Some(6)
```

La première ligne de `main` n'affiche rien du tout. Le compilateur le sait, et le dit :

```
warning: unused `Map` that must be used
 --> src/main.rs:4:5
  |
4 |     hotes.iter().map(|h| println!("sonde {h}"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: iterators are lazy and do nothing unless consumed
help: you might have meant to use `Iterator::for_each`
```

Pour un effet de bord, écris une boucle `for` (ou `for_each`, si tu tiens à la chaîne). Et regarde la suite : `find` demande un élément, `map` calcule pour `web-01`, la condition est vraie, et c'est fini. `db-01` et `cache-01` ne sont jamais calculés. C'est ce qui permet d'itérer sur un fichier de 10 Go ligne par ligne, ou sur une suite infinie (`(1..).map(...).take(5)`), sans rien charger d'avance.

### 5.11 Fermetures : ce qu'elles capturent

Une fermeture (*closure*) est une fonction anonyme qui peut utiliser les variables de l'endroit où elle est écrite : `|paramètres| expression`, ou `|paramètres| { bloc }`. Tu en as écrit une douzaine dans ce chapitre. Ce qui est propre à Rust, c'est la question du chapitre 4 : la fermeture **emprunte**-t-elle la variable, ou la **prend**-elle ?

```rust
use std::thread;

fn appliquer<F: Fn(u32) -> u32>(f: F, x: u32) -> u32 {
    f(x)
}

fn deux_fois<F: FnMut()>(mut f: F) {
    f();
    f();
}

fn une_fois<F: FnOnce() -> String>(f: F) -> String {
    f()
}

fn main() {
    let seuil = 50;
    let est_lent = |ms: u32| ms > seuil;
    println!("80 lent ? {}  seuil toujours là : {seuil}", est_lent(80));

    let mut appels = 0;
    let mut compter = || appels += 1;
    compter();
    compter();
    println!("appels = {appels}");

    let decalage = 1000;
    println!("{}", appliquer(|x| x + decalage, 22));

    let mut journal = Vec::new();
    deux_fois(|| journal.push("tic"));
    println!("{journal:?}");

    let nom = String::from("db-01");
    let message = une_fois(move || nom + " est tombé");
    println!("{message}");

    let hotes = vec!["web-01", "db-01"];
    let tache = thread::spawn(move || hotes.len());
    println!("le thread a compté {} hôtes", tache.join().unwrap());
}
```

```
$ cargo run
80 lent ? true  seuil toujours là : 50
appels = 2
1022
["tic", "tic"]
db-01 est tombé
le thread a compté 2 hôtes
```

Par défaut, le compilateur choisit la capture la plus légère qui suffit : `est_lent` lit `seuil`, donc l'emprunte (`&`), et `seuil` reste utilisable à côté. `compter` modifie `appels`, donc l'emprunte en `&mut`, ce qui exige `let mut compter` et interdit de lire `appels` tant que `compter` est encore utilisée. Le mot-clé **`move`** force la fermeture à prendre possession de ce qu'elle utilise : `nom` est déplacé dedans, et n'est plus utilisable après.

`move` est indispensable quand la fermeture vit plus longtemps que la fonction qui l'a créée : un thread, une tâche, une fermeture qu'on renvoie. Sans lui :

```
$ cargo build
error[E0373]: closure may outlive the current function, but it borrows `hotes`, which is owned by the current function
 --> src/main.rs:5:31
  |
5 |     let tache = thread::spawn(|| hotes.len());
  |                               ^^ ----- `hotes` is borrowed here
  |                               |
  |                               may outlive borrowed value `hotes`
  |
help: to force the closure to take ownership of `hotes` (and any other referenced variables), use the `move` keyword
```

Le thread peut tourner après la fin de `main`, et emprunter `hotes` serait garder une référence vers une variable morte. Le compilateur propose la correction exacte. Le chapitre 11 en fait son pain quotidien.

Les trois traits `Fn`, `FnMut` et `FnOnce` disent ce qu'une fonction attend d'une fermeture qu'on lui passe. `Fn` : appelable autant de fois qu'on veut sans rien modifier (un `filter`) ; `FnMut` : appelable plusieurs fois, peut modifier ce qu'elle a capturé (le `deux_fois` qui remplit `journal`) ; `FnOnce` : appelable une seule fois, parce qu'elle consomme ce qu'elle a capturé (`nom + "..."` déplace `nom`). Tu les liras dans les signatures bien avant d'en écrire, et le chapitre 7 explique la syntaxe `<F: Fn(u32) -> u32>`.

**Venant de Go :** une closure Go capture toujours par référence, et le ramasse-miettes garde la variable vivante aussi longtemps qu'il faut (chapitre 5 du cours Go). Rust n'a pas de ramasse-miettes, donc il faut choisir : emprunter (et le compilateur vérifie que la variable vit assez longtemps) ou déplacer (`move`). Le piège de la variable de boucle partagée, corrigé en Go 1.22, n'a jamais existé en Rust : chaque tour de boucle a sa variable, et une capture qui survivrait au tour ne compile pas.

**Venant du C :** une fermeture est une struct anonyme qui contient les variables capturées, plus une fonction qui la reçoit. C'est le couple `void (*f)(void *ctx), void *ctx` que tu passais à `qsort_r` ou à `pthread_create`, typé et vérifié.

### 5.12 Pour le labo

Le [labo 05](../labs/05-boite-a-outils/README.md) est une boîte à outils de sysadmin, et chaque fonction s'écrit en une ou deux chaînes d'itérateurs : compter les mots d'un texte (`HashMap` et `entry`), les N plus fréquents (tri stable par nombre décroissant puis par mot), dédoublonner en gardant l'ordre (`HashSet` et `retain`), des statistiques sur des latences (`total_cmp`, médiane), grouper des fichiers par extension (`BTreeMap`), analyser une ligne de log en une `struct` et compter les lignes par niveau. Les tests te diront si tu y es ; la solution montre chaque fois la chaîne la plus courte.

### À retenir

- `Vec<T>` est la collection par défaut ; `v[i]` panique hors bornes, `v.get(i)` rend un `Option`. Prends `Vec::with_capacity` quand tu connais la taille.
- `sort`, `sort_by_key` et `sort_by` sont stables ; `sort_unstable` est plus rapide quand l'ordre des égaux est indifférent ; un `f64` se trie avec `sort_by(f64::total_cmp)`.
- Une fonction qui lit une suite d'éléments prend `&[T]` (et `&str` pour du texte), jamais `&Vec<T>` ni `&String`. `chunks` et `windows` découpent sans copier.
- `HashMap` n'a pas d'ordre (graine aléatoire), `BTreeMap` itère dans l'ordre des clés ; `entry(k).or_insert(v)` et `or_default()` comptent et groupent en une ligne.
- `HashSet` pour les présences et les doublons, `VecDeque` pour une file.
- Une `String` est de l'UTF-8 dans un `Vec<u8>` : `len` compte les octets, `chars` les caractères, et `lines`, `trim`, `split_once`, `parse` font l'essentiel du travail sans copier.
- `iter()` donne des `&T`, `iter_mut()` des `&mut T`, `into_iter()` des `T` et consomme la collection.
- Les adaptateurs (`map`, `filter`, `filter_map`, `zip`...) sont paresseux ; un consommateur (`collect`, `sum`, `find`...) les fait tourner. `collect` a besoin d'un type cible, et un itérateur de `Result` se collecte en `Result<Vec<_>, _>`.
- Une fermeture emprunte ce qu'elle utilise ; `move` la force à en prendre possession, obligatoire pour un thread. `Fn`, `FnMut`, `FnOnce` disent combien de fois et comment on peut l'appeler.

---

← [4. Possession et emprunts : le cœur de Rust](04-possession-et-emprunts.md) · [Sommaire](../README.md) · [6. Structs, enums, méthodes, Option](06-structs-enums-option.md) →
