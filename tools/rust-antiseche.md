# Antisèche Rust 1.93

*La syntaxe sur deux pages. Les renvois « chap. N » pointent vers le [cours](../README.md). Tout ce qui est ici a été compilé avec Rust 1.93, édition 2024. Pour traduire depuis le C, Python ou Go, la [table de correspondance](c-python-go-vs-rust.md).*

## Squelette d'un programme et Cargo (chap. 1)

```rust
use std::env;                                // importer un chemin ; std est toujours là

fn main() {                                  // pas d'arguments ; retour () ou Result<(), E>
    let args: Vec<String> = env::args().skip(1).collect();   // args()[0] = le programme
    println!("Bonjour {args:?}");            // le ! : c'est une macro, format vérifié à la compilation
    if args.is_empty() {
        std::process::exit(1);               // code de retour ; les Drop en cours ne s'exécutent pas
    }
}
```

`cargo new sondes` (binaire, `src/main.rs`) ou `cargo new --lib sondes` (`src/lib.rs`) ; `cargo run -- arg1 arg2` ; `cargo build --release` (binaire dans `target/release/`) ; `cargo fmt` ; `cargo clippy`. Le dernier élément d'un bloc **sans** `;` est la valeur du bloc : `fn deux() -> i32 { 2 }`. `snake_case` pour les fonctions et variables, `CamelCase` pour les types et traits, `SCREAMING_SNAKE_CASE` pour les constantes. Tout est privé au module par défaut ; `pub` rend public.

## Variables, constantes, types de base (chap. 2)

```rust
let port = 5432;                             // immuable par défaut ; type déduit (i32 faute d'autre indice)
let mut essais: u32 = 0;                     // mut pour modifier ; annotation nom: Type
essais += 1;
let port = port.to_string();                 // masquage : nouvelle variable, le type peut changer
let (a, b) = (1, "deux");                    // déstructurer un tuple
let x;                                       // déclarer sans valeur ; lire avant d'affecter = E0381
x = 3;
const TIMEOUT_S: u64 = 5;                    // type obligatoire, évaluée à la compilation
static VERSION: &str = "1.2.0";              // une seule adresse pour tout le programme
let n = 1_000_000_i64;                       // suffixe de type ; 0xFF, 0o755, 0b1010, b'a' (un u8)
```

| Type | Exemples | Notes |
|---|---|---|
| `i8`…`i128`, `isize`, `u8`…`u128`, `usize` | `42`, `-1i8`, `1_000` | `i32` par défaut ; `usize` pour les index et `len()` ; débordement = **panic en debug**, modulo en release |
| `f32`, `f64` | `3.14`, `1e-9` | `f64` par défaut ; `f64::NAN` ; pas `Ord` à cause de NaN, donc `v.sort()` refuse un `Vec<f64>` : `v.sort_by(f64::total_cmp)` |
| `bool` | `true`, `false` | aucune conversion : `if n {` refuse un entier |
| `char` | `'a'`, `'é'`, `'\u{1F98A}'` | 4 octets, une valeur Unicode, jamais un octet |
| `&str` | `"web-01"`, `r"C:\brut"`, `r#"{"a": 1}"#` | tranche de chaîne **empruntée**, UTF-8 ; un littéral est un `&'static str` |
| `String` | `String::from("a")`, `"a".to_string()`, `format!(…)` | chaîne **possédée**, sur le tas, modifiable |
| `()` | `()` | « unit » : ce que renvoie une fonction qui ne renvoie rien |
| `(T, U)`, `[T; N]`, `&[T]`, `Vec<T>`, `HashMap<K, V>`, `Option<T>`, `Result<T, E>`, `Box<T>` | | chap. 4, 5, 6, 8, 10 |
| `!` | `panic!()`, `loop {}`, `std::process::exit(1)` | une expression qui ne revient jamais |

Conversions **toujours explicites**. Avec perte possible : `n as f64`, `f as i32` (tronque et sature), `300_i32 as u8` (coupe : 44). Sans perte : `i64::from(n)`, `let x: u64 = n.into()`. Qui peut échouer : `u8::try_from(300)` renvoie `Err`. Chaîne vers nombre : `"42".parse::<i32>()` renvoie un `Result` ; nombre vers chaîne : `n.to_string()`. Arithmétique choisie : `a.checked_add(b)` (`Option`), `a.saturating_sub(b)`, `a.wrapping_mul(b)`.

## Chaînes et format! (chap. 2)

```rust
let s = "web-01.stranix.net";                                  // &str
let mut o = String::from("pg");                                // String
o.push('-'); o.push_str("01"); o += ".lan";                    // ajouter ; s1 + &s2 consomme s1
s.len(); s.chars().count(); &s[0..6];                          // octets ; caractères ; tranche par octets (panic au milieu d'un caractère)
s.contains("web"); s.starts_with("web"); s.ends_with(".net"); s.find('.');   // find -> Option<usize>
s.split('.').collect::<Vec<_>>(); s.split_once('.'); s.trim(); s.to_uppercase(); s.replace('-', "_");
s.split_whitespace(); s.lines(); s.strip_prefix("web-");       // strip_prefix -> Option<&str>
for (i, c) in s.char_indices() {}                              // caractères et index d'octet ; s.bytes() pour les octets
let parts = ["a", "b"].join("."); let n: i32 = "42".parse()?;  // parse -> Result, ? remonte l'erreur
let (h, p, v, f) = ("web", 22, vec![1, 2], 3.14159);
let texte = format!("{h}:{p} {v:?} {v:#?} {f:.2} {h:>10} {h:<10} {p:05} {p:x} {p:#b} {{}}");
```

`{}` demande le trait `Display`, `{:?}` le trait `Debug` (`#[derive(Debug)]`), `{:#?}` le Debug indenté. `{nom}` capture une variable, pas une expression : `{h.nom}` ne compile pas, écris `{}` et passe `h.nom`. Les cousins : `println!`, `print!`, `eprintln!` (stderr), `write!`/`writeln!` (vers un `Write`), `panic!`. En paramètre, prends `&str` ; en champ de struct ou en retour, une `String`.

## Contrôle et match (chap. 3)

```rust
let v = vec![3, 1, 2]; let mut pile = v.clone(); let n = 7;
let signe = if n < 0 { "négatif" } else { "positif" };          // if est une expression : pas de ternaire
if n > 0 && n < 10 || n == 42 {}                                // && || ! ; une condition bool, sans parenthèses
let mut i = 0;
while i < 3 { i += 1; }
for i in 0..10 {}                                               // 0 à 9 ; 0..=10 inclut 10 ; (0..10).rev(), .step_by(2)
for x in &v {}                                                  // emprunte v ; `for x in v` le consomme ; &mut v pour modifier
let trouve = loop { i += 1; if i == 5 { break i * 2; } };       // loop renvoie une valeur par break
'externe: for a in 0..3 { for b in 0..3 { if a * b == 2 { break 'externe; } } }   // étiquette
let etat = match n {                                            // exhaustif : un cas oublié = E0004
    0 => "zéro",
    1 | 2 => "petit",                                           // plusieurs motifs
    3..=9 => "moyen",                                           // intervalle
    x if x % 2 == 0 => "grand et pair",                         // garde
    _ => "autre",                                               // le joker
};
if let Some(x) = v.first() {}                                   // un seul motif
if let Some(x) = v.first() && *x > 2 {}                         // let chains : édition 2024, Rust 1.88
while let Some(haut) = pile.pop() {}                            // tant que le motif colle
let Some(premier) = v.first() else { return; };                 // let-else : sortir si le motif rate
```

`match` marche sur les entiers, les chaînes (`match s.as_str()`), les tuples, les enums, les tranches (`[premier, reste @ ..]`) et les structs. Pas de `++`, pas de `do while` (`loop { …; if !c { break; } }`), pas de `goto`.

## Possession, emprunts, tranches, durées de vie (chap. 4)

```rust
let a = String::from("web-01");
let b = a;                                   // DÉPLACEMENT : a n'est plus utilisable (E0382 borrow of moved value)
let c = b.clone();                           // copie profonde, explicite et visible
let n = 5; let m = n;                        // types Copy (entiers, f64, bool, char, &T) : copiés, n reste valide
fn longueur(s: &str) -> usize { s.len() }    // emprunt partagé : lire sans prendre
fn ajoute(s: &mut String) { s.push('!'); }   // emprunt exclusif : modifier sans prendre
fn consomme(s: String) {}                    // prend la possession : s est libérée à la fin de la fonction
let mut d = c.clone();
longueur(&d); ajoute(&mut d); consomme(d);   // &d, &mut d à l'appel : on voit qui fait quoi
let v = vec![1, 2, 3, 4];
let t: &[i32] = &v[1..3];                    // tranche : vue empruntée sur une partie, ici [2, 3]
let mot: &str = &c[..3];                     // un &str est une tranche de String
fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }   // le résultat vit autant que le plus court des deux
```

La règle : à tout instant, **soit** autant de `&T` que tu veux, **soit** un seul `&mut T`, et une référence ne survit jamais à la valeur (E0499 deux `&mut`, E0502 `&` et `&mut` mélangés, E0597 `does not live long enough`). La valeur est libérée quand son propriétaire sort de portée, sans ramasse-miettes. Les durées de vie s'écrivent rarement : un seul paramètre référence, ou `&self`, et le compilateur les déduit. `'static` = vit tout le programme. En paramètre, préfère `&str` à `&String` et `&[T]` à `&Vec<T>` : ça accepte plus de choses.

## Vec, HashMap, itérateurs, fermetures (chap. 5)

```rust
use std::collections::{HashMap, HashSet};
let mut v: Vec<i32> = Vec::new(); let mut w = vec![3, 1, 2]; let z: Vec<u8> = Vec::with_capacity(10);
v.push(4); v.pop(); v.extend([5, 6]); v.insert(0, 9); v.remove(0); v.len(); v.is_empty();
w[0]; w.get(10);                                                   // w[10] panique ; get -> Option<&T>
w.sort(); w.sort_by_key(|x| -x); w.dedup(); w.contains(&3); w.iter().position(|&x| x == 3);
w.retain(|&x| x > 1); w.truncate(1); w.clear();
let mut m: HashMap<String, u16> = HashMap::new();                 // BTreeMap : même API, clés triées
m.insert("web-01".into(), 22); m.get("web-01"); m.contains_key("x"); m.remove("x");   // get -> Option<&V>
*m.entry("pg-01".to_string()).or_insert(0) += 1;                  // compter : l'idiome entry
for (k, val) in &m {}                                             // ordre ALÉATOIRE
let s: HashSet<&str> = ["a", "b"].into_iter().collect();          // ensemble ; VecDeque pour une file
let double = |x: i32| x * 2;                                      // fermeture : capture l'environnement par emprunt
let mut total = 0; let mut cumule = |x: i32| total += x; cumule(3);   // FnMut : capture mutable
let seuil = 2; let garde = move |x: &i32| *x > seuil;             // move : prend la possession des captures (threads)
let impairs: Vec<i32> = (1..=5).map(|x| x * x).filter(|x| x % 2 == 1).collect();   // rien ne tourne avant collect
let somme: i32 = impairs.iter().sum();
```

| Adaptateur (paresseux) | Fait | | Consommateur | Fait |
|---|---|---|---|---|
| `iter()`, `iter_mut()`, `into_iter()` | donne `&T`, `&mut T`, `T` | | `collect::<Vec<_>>()` | vers `Vec`, `HashMap`, `String`, `Result<Vec<_>, _>` |
| `map(f)`, `filter(p)` | transformer, garder | | `sum()`, `product()`, `count()` | type de retour à annoter |
| `filter_map(f)`, `flat_map(f)`, `flatten()` | garder les `Some`, aplatir | | `min()`, `max()`, `max_by_key(f)`, `min_by(f)` | renvoient `Option` |
| `enumerate()`, `zip(b)`, `chain(b)` | `(i, x)`, paires, à la suite | | `fold(init, f)`, `reduce(f)` | accumuler |
| `skip(n)`, `take(n)`, `skip_while(p)`, `take_while(p)`, `step_by(n)` | découper | | `any(p)`, `all(p)`, `find(p)`, `position(p)` | court-circuitent |
| `rev()`, `peekable()`, `cloned()`, `copied()`, `inspect(f)` | retourner, regarder, copier | | `last()`, `nth(n)`, `for_each(f)` | |
| `windows(2)`, `chunks(3)`, `chunk_by(f)` (sur une tranche) | fenêtres, paquets | | `partition(p)`, `unzip()` | deux collections |

## Structs, enums, Option (chap. 6)

```rust
#[derive(Debug, Clone, PartialEq, Default)]                  // génère {:?}, .clone(), ==, Hote::default()
struct Hote { nom: String, port: u16, tags: Vec<String> }
impl Hote {
    fn new(nom: &str) -> Self { Self { nom: nom.to_string(), port: 22, ..Default::default() } }   // « constructeur » : simple fonction associée
    fn adresse(&self) -> String { format!("{}:{}", self.nom, self.port) }   // &self : lit
    fn renomme(&mut self, n: &str) { self.nom = n.into(); }               // &mut self : modifie
    fn detruit(self) -> String { self.nom }                               // self : consomme la valeur
}
let mut h = Hote::new("web-01"); h.renomme("web-02"); let pg = Hote { port: 5432, ..h.clone() };
struct Port(u16);                                            // tuple struct : un nouveau type (newtype), champ .0
enum Etat { Up, Down { depuis: u64 }, Inconnu(String) }      // chaque variante porte ses propres données
let e = Etat::Down { depuis: 30 };
match e { Etat::Up => {} Etat::Down { depuis } if depuis > 60 => {} Etat::Down { .. } => {} Etat::Inconnu(msg) => {} }
let o: Option<u16> = Some(22);                               // Some(v) ou None : la place de nil, NULL, None
```

| Méthode d'`Option<T>` | Pour |
|---|---|
| `o.is_some()`, `o.is_none()`, `o.is_some_and(\|p\| p > 1024)` | tester |
| `if let Some(p) = o`, `let Some(p) = o else { … }`, `match o` | déballer proprement |
| `o?` | renvoyer `None` tout de suite (dans une fonction qui renvoie `Option`) |
| `o.unwrap()`, `o.expect("port absent")` | la valeur ou panic : tests et prototypes seulement |
| `o.unwrap_or(80)`, `o.unwrap_or_default()`, `o.unwrap_or_else(\|\| calcule())` | une valeur de repli |
| `o.map(\|p\| p + 1)`, `o.and_then(\|p\| p.checked_add(1))`, `o.filter(\|p\| *p > 1024)` | transformer sans déballer |
| `o.ok_or("pas de port")`, `o.ok_or_else(\|\| …)` | vers `Result`, pour enchaîner avec `?` |
| `o.as_ref()`, `o.as_deref()`, `o.as_mut()` | `Option<&T>` ; `Option<&str>` depuis `Option<String>` |
| `o.take()`, `o.replace(v)`, `o.get_or_insert(v)`, `o.insert(v)` | vider ou remplir un `Option` en place |
| `o.or(Some(80))`, `o.xor(b)`, `o.zip(b)` | combiner |
| `v.get(0).copied()`, `.cloned()` | `Option<&T>` vers `Option<T>` (ce que renvoient `get`, `first`, `iter().max()`) |

## Traits, génériques, dyn (chap. 7)

```rust
trait Sonde {
    fn nom(&self) -> String;                                         // à implémenter
    fn decrire(&self) -> String { format!("sonde {}", self.nom()) }  // méthode par défaut
}
struct Tcp { port: u16 }
impl Sonde for Tcp { fn nom(&self) -> String { format!("tcp/{}", self.port) } }   // implémentation EXPLICITE
fn lance<T: Sonde>(s: &T) -> String { s.decrire() }                 // générique : une copie compilée par type
fn lance_court(s: &impl Sonde) -> String { s.nom() }                // la même chose, écrite court
fn plus_grand<T>(a: T, b: T) -> T where T: PartialOrd { if a > b { a } else { b } }   // where pour les bornes longues
let sondes: Vec<Box<dyn Sonde>> = vec![Box::new(Tcp { port: 22 })];   // dyn : appel résolu à l'exécution, types mélangés
fn fabrique() -> impl Iterator<Item = u32> { (1..4).map(|x| x * 10) }  // impl Trait en retour : type concret caché
impl std::fmt::Display for Tcp {                                    // permet {} et .to_string()
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "tcp:{}", self.port) }
}
```

| Trait de la std | Donne | Comment l'avoir |
|---|---|---|
| `Debug` | `{:?}`, `dbg!(x)` | `#[derive(Debug)]` sur presque tout |
| `Display` | `{}`, `.to_string()` | à écrire (`impl fmt::Display`) |
| `Clone`, `Copy` | `.clone()` explicite ; copie implicite à l'affectation | derive ; `Copy` seulement si tous les champs le sont (pas de `String`, pas de `Vec`) |
| `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` | `==`, `<`, `sort()`, clé de `HashMap` | derive ; `f64` n'est que `PartialEq` et `PartialOrd` |
| `Default` | `T::default()`, `..Default::default()`, `unwrap_or_default()` | derive : 0, `""`, vide, `None` |
| `From`, `Into` | `String::from("a")`, `let s: String = "a".into()` | écris `From`, `Into` vient gratuitement ; `?` s'en sert pour convertir les erreurs |
| `TryFrom`, `TryInto`, `FromStr` | `u8::try_from(300)`, `"42".parse::<T>()` | conversions qui peuvent échouer |
| `AsRef<str>`, `AsRef<Path>` | accepter `&str`, `String`, `PathBuf` sans choisir | `fn ouvre(p: impl AsRef<Path>)` |
| `Deref` | `*x`, et `&String` qui passe pour un `&str` | `Box`, `String`, `Vec`, `Rc` l'ont |
| `Drop` | du code à la destruction | `impl Drop` ; jamais appelé à la main (`drop(x)` déplace et libère) |
| `Iterator`, `IntoIterator` | `for`, tous les adaptateurs | une seule méthode : `fn next(&mut self) -> Option<Self::Item>` |
| `Error` | une erreur qu'on peut chaîner et afficher | `Debug + Display` ; `#[derive(thiserror::Error)]` |
| `Read`, `Write`, `BufRead` | flux d'octets, `lines()` | `use std::io::{Read, Write, BufRead}` |
| `Send`, `Sync` | traverser un thread, se partager entre threads | automatiques, chap. 11 |

## Result, ?, thiserror, anyhow (chap. 8)

```rust
use std::fs;
use std::num::ParseIntError;
fn lire_port(s: &str) -> Result<u16, ParseIntError> { s.trim().parse::<u16>() }   // Ok(v) ou Err(e)
fn config(chemin: &str) -> Result<u16, Box<dyn std::error::Error>> {   // n'importe quelle erreur, sans crate
    let texte = fs::read_to_string(chemin)?;                 // ? : si Err, convertit (From) et renvoie tout de suite
    Ok(lire_port(&texte)?)
}
match lire_port("80") { Ok(p) => println!("{p}"), Err(e) => eprintln!("erreur : {e}") }
#[derive(Debug, thiserror::Error)]                           // bibliothèque : erreurs typées, l'appelant peut les distinguer
enum ErreurConfig {
    #[error("clé manquante : {0}")]
    Manquante(String),
    #[error("lecture de {chemin}")]
    Lecture { chemin: String, #[source] source: std::io::Error },
    #[error(transparent)]
    Port(#[from] ParseIntError),                             // #[from] : ? convertit tout seul
}
fn appli() -> anyhow::Result<()> {                           // application : une seule erreur, du contexte
    use anyhow::{Context, bail};
    let texte = fs::read_to_string("app.toml").context("lecture de app.toml")?;
    if texte.is_empty() { bail!("app.toml est vide"); }      // bail! = return Err(anyhow!(…))
    Ok(())
}
```

| Méthode de `Result<T, E>` | Pour |
|---|---|
| `r?` | renvoyer l'erreur à l'appelant, convertie par `From` |
| `r.is_ok()`, `r.is_err()`, `r.as_ref().is_ok_and(\|v\| …)` | tester (`is_ok_and` consomme `r` : d'où le `as_ref()`) |
| `r.unwrap()`, `r.expect("config lisible")` | la valeur ou panic |
| `r.unwrap_or(0)`, `r.unwrap_or_default()`, `r.unwrap_or_else(\|e\| …)` | une valeur de repli |
| `r.map(f)`, `r.map_err(\|e\| …)`, `r.and_then(f)`, `r.or_else(f)` | transformer la valeur ou l'erreur |
| `r.ok()`, `r.err()` | vers `Option` |
| `r.context("…")`, `r.with_context(\|\| format!(…))` | anyhow : ajouter du contexte (marche aussi sur `Option`) |
| `iter.collect::<Result<Vec<_>, _>>()` | tout ou la première erreur |
| `panic!`, `unreachable!`, `todo!`, `assert!`, `assert_eq!`, `debug_assert!` | les bugs, pas les erreurs attendues |

`thiserror` dans une bibliothèque, `anyhow` dans un `main`. `{e}` affiche le message, `{e:?}` le détail ; avec anyhow, `{e:#}` affiche toute la chaîne de causes sur une ligne. `main` peut renvoyer `anyhow::Result<()>` : une erreur sort avec le code 1.

## Modules, Cargo.toml, tests, doc (chap. 9)

````rust
mod reseau {                                     // en ligne ; ou un fichier reseau.rs déclaré par `mod reseau;`
    pub struct Cible { pub nom: String, port: u16 }   // pub champ par champ : port reste privé
    pub fn sonder(c: &Cible) -> bool { c.port > 0 }
    pub(crate) fn interne() {}                   // visible dans toute la crate, pas dehors
    pub mod tcp { pub fn ouvrir() { super::interne(); } }   // super = parent ; crate:: = racine
}
use reseau::{Cible, sonder};                     // raccourcir ; pub use pour réexporter
// /// documente le symbole qui suit, en Markdown ; un bloc de code dedans est un test (cargo test --doc)
/// Additionne deux ports.
///
/// ```
/// assert_eq!(2 + 3, 5);
/// ```
pub fn somme(a: u16, b: u16) -> u16 { a + b }
#[cfg(test)]                                     // compilé seulement par cargo test
mod tests {
    use super::*;
    #[test]
    fn somme_simple() { assert_eq!(somme(2, 3), 5, "2 + 3"); }   // assert!, assert_ne!
    #[test]
    #[should_panic(expected = "overflow")]
    fn deborde() { somme(u16::MAX, std::hint::black_box(1)); }   // un débordement panique en debug
    #[test]
    fn avec_result() -> Result<(), String> { Ok(()) }            // un test peut renvoyer Result et utiliser ?
}
````

```toml
[package]
name = "sondes"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"                            # version minimale de Rust annoncée

[dependencies]
serde = { version = "1", features = ["derive"] } # "1" = ^1 : toute 1.x compatible
tokio = { version = "1", features = ["full"] }

[dev-dependencies]                               # seulement pour les tests et exemples
tempfile = "3"
```

Disposition : `src/main.rs` (binaire), `src/lib.rs` (bibliothèque, ce que les tests d'intégration voient), `src/bin/autre.rs` (binaires en plus), `tests/*.rs` (tests d'intégration, une crate chacun), `examples/`, `benches/`. `Cargo.lock` se committe pour un binaire. Un `[workspace]` à la racine partage un seul `target/` entre plusieurs paquets. `#[ignore]` met un test de côté (`cargo test -- --ignored`).

## Box, Rc, RefCell, Arc, unsafe (chap. 10)

```rust
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
let b: Box<i32> = Box::new(5);                       // sur le tas, un seul propriétaire ; libéré en sortie de portée
enum Liste { Noeud(i32, Box<Liste>), Fin }           // type récursif : Box lui donne une taille connue
let partage = Rc::new(String::from("config"));       // plusieurs propriétaires, un seul thread
let autre = Rc::clone(&partage);                     // incrémente le compteur, ne copie pas la chaîne
let nb = Rc::strong_count(&partage);                 // 2
let cellule = RefCell::new(vec![1]);                 // mutabilité intérieure : emprunts vérifiés à l'exécution
cellule.borrow_mut().push(2);                        // deux borrow_mut vivants en même temps = panic
let commun = Rc::new(RefCell::new(0)); *commun.borrow_mut() += 1;   // le couple classique, un thread
let entre_threads = Arc::new(Mutex::new(0));         // l'équivalent multi-thread (chap. 11)
struct Garde;
impl Drop for Garde { fn drop(&mut self) { println!("libéré"); } }   // à la destruction, ordre inverse de création
let x = 42; let p = &x as *const i32;                // créer un pointeur brut est sûr
let lu = unsafe { *p };                              // le déréférencer demande unsafe : c'est toi qui garantis
```

| Pointeur | Quand | Coût, piège |
|---|---|---|
| `&T`, `&mut T` | par défaut : utiliser sans posséder | nul ; vérifié à la compilation |
| `Box<T>` | un seul propriétaire, sur le tas : type récursif, gros objet, `Box<dyn Trait>` | un pointeur ; rien d'autre |
| `Rc<T>` | plusieurs propriétaires, un seul thread : arbre, graphe, config partagée | compteur ; pas `Send` (E0277 si tu l'envoies dans un thread) ; cycles = fuite |
| `Arc<T>` | plusieurs propriétaires, plusieurs threads | compteur atomique ; lecture seule sans `Mutex` |
| `Cell<T>` | modifier un petit `Copy` derrière un `&` | pas de référence vers l'intérieur |
| `RefCell<T>` | modifier derrière un `&`, un seul thread : `Rc<RefCell<T>>` | vérifié à l'exécution : panic si deux `borrow_mut` |
| `Mutex<T>`, `RwLock<T>` | modifier entre threads : `Arc<Mutex<T>>` | verrou ; libéré quand le garde est détruit |
| `Weak<T>` | pointer sans retenir : lien vers le parent, cache | `upgrade()` renvoie `Option` |
| `Cow<'a, str>` | emprunter si possible, posséder si nécessaire | rare, pour éviter des copies |
| `*const T`, `*mut T` | FFI, structures bas niveau | aucune garantie, `unsafe` pour lire |

`unsafe` ne coupe pas le vérificateur d'emprunts : il autorise cinq choses de plus (déréférencer un pointeur brut, appeler une fonction `unsafe`, accéder à un `static mut`, implémenter un trait `unsafe`, lire une `union`). Édition 2024 : `std::env::set_var` est `unsafe`, et un bloc `extern` s'écrit `unsafe extern "C" { … }`.

## Threads, mpsc, Mutex, atomiques (chap. 11)

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock, mpsc};
use std::thread;
let h = thread::spawn(|| 2 + 2);                     // un vrai thread de l'OS
let quatre = h.join().unwrap();                      // join attend ; Err si le thread a paniqué
let noms = vec!["web-01", "pg-01"];
thread::scope(|s| {                                  // threads à portée : ils peuvent emprunter noms
    for n in &noms { s.spawn(move || println!("{n}")); }
});                                                  // attend tous les threads ici
let (tx, rx) = mpsc::channel::<String>();            // plusieurs émetteurs, un récepteur ; sync_channel(10) : borné
for i in 0..3 { let tx = tx.clone(); thread::spawn(move || tx.send(format!("r{i}")).unwrap()); }
drop(tx);                                            // détruire l'original, sinon la boucle ne finit jamais
for msg in rx {}                                     // s'arrête quand tous les émetteurs sont détruits
let compteur = Arc::new(Mutex::new(0));              // partager ET modifier : Arc<Mutex<T>>
let c = Arc::clone(&compteur);
thread::spawn(move || *c.lock().unwrap() += 1).join().unwrap();   // lock() rend un garde, libéré à sa destruction
let config = RwLock::new(String::new()); config.read().unwrap(); config.write().unwrap().push('x');
static REQUETES: AtomicU64 = AtomicU64::new(0);
REQUETES.fetch_add(1, Ordering::Relaxed);            // compteur sans verrou ; load(Ordering::Relaxed) pour lire
static HOTE: LazyLock<String> = LazyLock::new(|| std::env::var("HOTE").unwrap_or_default());   // initialisé au premier accès
```

Un thread coûte de l'ordre d'une pile de 2 Mo : quelques centaines, pas cent mille (pour ça, des tâches tokio, chap. 13). Le compilateur refuse la course de données : `Rc` ou `RefCell` dans `thread::spawn`, c'est `error[E0277]: ... cannot be sent between threads safely`. `lock().unwrap()` panique si un autre thread a paniqué en tenant le verrou (« empoisonné »).

## Fichiers, Command, clap, serde (chap. 12)

```rust
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
let texte = fs::read_to_string("sondes.json")?; fs::write("copie.json", &texte)?;   // petits fichiers, d'un coup
let f = fs::File::open("hotes.txt")?;
for ligne in BufReader::new(f).lines() { let ligne = ligne?; }                     // ligne par ligne
let mut rapport = fs::File::create("rapport.txt")?; writeln!(rapport, "ok")?;      // OpenOptions::new().append(true) pour ajouter
let p: PathBuf = Path::new("/etc").join("hosts"); p.exists(); p.extension(); fs::create_dir_all("out")?;
for entree in fs::read_dir(".")? { let chemin = entree?.path(); }
let dsn = std::env::var("DSN").unwrap_or_default();                                 // var -> Result
let sortie = Command::new("pg_dump").args(["-h", "localhost", "base"]).output()?;  // pas de shell : pas d'injection
let ok = sortie.status.success(); let texte = String::from_utf8_lossy(&sortie.stdout);
```

```rust
use clap::Parser;
use serde::{Deserialize, Serialize};
#[derive(Parser)]                                    // les options sortent de la struct
#[command(version, about = "Sonde des hôtes")]
struct Args {
    fichier: String,                                 // argument positionnel obligatoire
    #[arg(short, long, default_value_t = 5)]
    timeout: u64,                                    // -t 5 ou --timeout 5
    #[arg(short, long)]
    verbeux: bool,                                   // drapeau -v
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]                   // noms des champs côté JSON
struct Hote {
    nom: String,
    #[serde(default)]
    port: u16,                                       // absent du JSON : 0
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,                            // null ou absent : None
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();                        // --help et --version offerts
    let hotes: Vec<Hote> = serde_json::from_str(&std::fs::read_to_string(&args.fichier)?)?;
    println!("{}", serde_json::to_string_pretty(&hotes)?);   // to_string, to_writer, from_reader
    let brut: serde_json::Value = serde_json::json!({ "nom": "web-01", "port": 22 });   // JSON sans struct
    Ok(())
}
```

## async, tokio, axum, reqwest (chap. 13)

```rust
use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};
use std::sync::{Arc, Mutex};
use std::time::Duration;
type Etat = Arc<Mutex<Vec<String>>>;
async fn liste(State(etat): State<Etat>) -> Json<Vec<String>> { Json(etat.lock().unwrap().clone()) }
async fn une(Path(id): Path<usize>, State(etat): State<Etat>) -> Result<String, StatusCode> {
    etat.lock().unwrap().get(id).cloned().ok_or(StatusCode::NOT_FOUND)   // jamais un Mutex std tenu pendant un .await
}
#[tokio::main]                                       // démarre le runtime ; main devient async
async fn main() -> anyhow::Result<()> {
    let t = tokio::spawn(async { 42 });              // tâche légère, l'équivalent de go f()
    let r = tokio::time::timeout(Duration::from_secs(5), t).await??;   // 1er ? : délai dépassé, 2e : tâche paniquée
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;   // un client, réutilisé
    let corps = client.get("https://example.com").send().await?.error_for_status()?.text().await?;   // .json::<T>().await? aussi
    let (a, b) = tokio::join!(async { 1 }, async { 2 });   // attendre plusieurs futures en même temps
    tokio::select! {                                 // le premier prêt gagne, les autres sont abandonnés
        _ = tokio::time::sleep(Duration::from_millis(10)) => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    let app = Router::new().route("/taches", get(liste)).route("/taches/{id}", get(une)).with_state(Etat::default());
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(ecoute, app).await?;                 // .with_graceful_shutdown(signal) pour s'arrêter proprement
    Ok(())
}
```

Une fonction `async` ne fait **rien** tant qu'on ne l'attend pas (`.await`) ou qu'on ne la lance pas (`tokio::spawn`). Jamais de code bloquant (`std::thread::sleep`, `std::fs` lourd, calcul long) dans une tâche : `tokio::time::sleep`, `tokio::fs`, `tokio::task::spawn_blocking`. Pour garder un verrou à travers un `.await` : `tokio::sync::Mutex`. Channels async : `tokio::sync::mpsc`, `oneshot`, `broadcast`, `watch`.

## rusqlite et postgres (chap. 14)

```rust
use rusqlite::{Connection, OptionalExtension, params};
let mut conn = Connection::open_in_memory()?;                    // ou Connection::open("inventaire.db")
conn.execute("CREATE TABLE hotes (nom TEXT PRIMARY KEY, port INTEGER)", [])?;
conn.execute("INSERT INTO hotes (nom, port) VALUES (?1, ?2)", params!["web-01", 22])?;   // paramètres liés, jamais format!
let port: Option<u16> = conn.query_row("SELECT port FROM hotes WHERE nom = ?1", ["web-01"], |l| l.get(0)).optional()?;
let mut req = conn.prepare("SELECT nom, port FROM hotes")?;
let hotes = req.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, u16>(1)?)))?.collect::<Result<Vec<_>, _>>()?;
drop(req);                                                        // la requête emprunte conn : la rendre avant transaction()
let tx = conn.transaction()?; tx.execute("DELETE FROM hotes", [])?; tx.commit()?;   // sans commit : rollback au Drop
```

```rust
let (client, connexion) = tokio_postgres::connect("host=localhost user=postgres dbname=app", tokio_postgres::NoTls).await?;
tokio::spawn(connexion);                                          // la connexion vit dans sa propre tâche
let lignes = client.query("SELECT nom, port FROM hotes WHERE port > $1", &[&1024_i32]).await?;
for l in &lignes { let nom: String = l.get("nom"); let port: i32 = l.get(1); }   // INTEGER = i32 ; mauvais type = panic
let une = client.query_opt("SELECT nom FROM hotes WHERE nom = $1", &[&"web-01"]).await?;   // Option<Row>
let n = client.execute("UPDATE hotes SET port = $1 WHERE nom = $2", &[&5432_i32, &"pg-01"]).await?;   // lignes touchées
```

Les types SQL se lisent strictement : `INTEGER` en `i32`, `BIGINT` en `i64`, `TEXT` en `String`, `NULL` en `Option<T>`. Pour un pool : `deadpool-postgres` ou `sqlx::PgPool`.

## clippy, profils, cross, Docker (chap. 15)

```toml
[profile.release]
lto = true                                       # optimisation à l'édition de liens : plus lent à compiler, plus rapide
codegen-units = 1                                # un seul morceau : binaire plus petit
strip = true                                     # retire les symboles
panic = "abort"                                  # pas de déroulement de pile : plus petit, catch_unwind inutile
```

```dockerfile
FROM rust:1.93 AS build
WORKDIR /src
COPY . .
RUN rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl
FROM scratch
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/sondes /sondes
ENTRYPOINT ["/sondes"]
```

Avec musl le binaire est statique et tourne dans `scratch` ; une crate qui compile du C (`rusqlite` bundled, `ring`) veut en plus `musl-tools` dans l'image de build. Sur un Mac Apple Silicon, `cross build --target x86_64-unknown-linux-musl` (via Docker) ou `cargo zigbuild` évitent de monter une chaîne croisée. `#[expect(clippy::too_many_arguments)]` fait taire un lint ponctuellement, et râle si le lint ne se déclenche plus.

## Bibliothèque standard et crates à connaître

| Module ou crate | Pour |
|---|---|
| `std::fmt`, `std::str`, `std::string`, `std::char` | texte |
| `std::collections` (`HashMap`, `BTreeMap`, `HashSet`, `VecDeque`, `BinaryHeap`) | données |
| `std::fs`, `std::io`, `std::path`, `std::env`, `std::process` | fichiers, flux, processus |
| `std::thread`, `std::sync` (`Arc`, `Mutex`, `mpsc`, `atomic`, `LazyLock`, `OnceLock`) | concurrence |
| `std::time` (`Duration`, `Instant`, `SystemTime`), `std::net` (`TcpStream`, `IpAddr`) | temps, réseau |
| `std::rc`, `std::cell`, `std::mem`, `std::ptr`, `std::hint` | sous le capot |
| `serde`, `serde_json`, `toml`, `clap` | formats, ligne de commande |
| `thiserror`, `anyhow` | erreurs |
| `tokio`, `reqwest`, `axum`, `tower` | async, HTTP |
| `rusqlite`, `tokio-postgres`, `sqlx` | bases de données |
| `tracing`, `tracing-subscriber`, `regex`, `chrono` ou `jiff`, `rand`, `rayon`, `tempfile` | journaux, motifs, dates, aléa, parallélisme, tests |

## Commandes cargo

| Commande | Fait |
|---|---|
| `cargo new nom`, `cargo new --lib nom`, `cargo init` | créer un paquet |
| `cargo add serde --features derive`, `cargo add --dev tempfile`, `cargo remove x` | gérer les dépendances (écrit dans `Cargo.toml`) |
| `cargo check` | vérifier sans produire de binaire : la boucle rapide |
| `cargo build`, `cargo build --release` | `target/debug/` ou `target/release/` |
| `cargo run -- --port 8080`, `cargo run --bin autre`, `cargo run --example demo` | lancer ; les arguments après `--` |
| `cargo test`, `cargo test nom`, `cargo test -- --nocapture`, `cargo test --doc` | tests ; filtre par nom ; voir les `println!` |
| `cargo fmt`, `cargo fmt --check` | formater ; vérifier en CI |
| `cargo clippy --all-targets -- -D warnings` | lint, avertissements bloquants |
| `cargo doc --open`, `cargo tree`, `cargo update`, `cargo clean` | doc locale, arbre des dépendances, mettre à jour le lock, vider `target/` |
| `cargo build --release --target aarch64-unknown-linux-gnu` | compilation croisée, après `rustup target add …` ; liste : `rustup target list` |
| `cargo install ripgrep`, `cargo install --path .` | installer un binaire dans `~/.cargo/bin` |
| `rustup update`, `rustup component add clippy rustfmt`, `rustc --explain E0382` | outillage ; explication longue d'une erreur |
