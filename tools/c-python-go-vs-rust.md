# C / Python / Go / Rust : la table de correspondance

*Pour qui vient du C, de Python et de Go et cherche « comment on dit en Rust ». La colonne **Piège** signale ce qui se comporte autrement qu'attendu quand on traduit mot à mot, surtout depuis Go. Renvois vers le [cours](../README.md) et l'[antisèche](rust-antiseche.md).*

## Déclarer, typer, nommer

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Déclarer une variable | `int n = 5;` | `n = 5` | `n := 5` ou `var n int = 5` | `let n = 5;` ou `let n: i64 = 5;` | `let` est **immuable** : `n += 1` donne `error[E0384]: cannot assign twice to immutable variable`, il faut `let mut n`. Redéclarer avec `let` dans le même bloc est permis et courant (masquage), là où Go refuse un `:=` sans nouvelle variable (chap. 2). |
| Sans valeur initiale | `int n;` (indéfini) | n'existe pas | `var n int` (vaut 0) | `let n: i32;` puis `n = 5;` une fois | pas de valeur zéro implicite : lire avant d'affecter, c'est `error[E0381]: used binding isn't initialized`. Une variable inutilisée n'est qu'un avertissement (pas une erreur comme en Go) ; `_n` le fait taire. |
| Valeur zéro | non | non | toute variable a sa valeur zéro | `#[derive(Default)]` puis `Hote::default()` | **opt-in** : une struct sans `Default` n'en a pas. `Hote { nom: "a".into(), ..Default::default() }` remplace le littéral partiel `Hote{Nom: "a"}` de Go. |
| Constante | `#define N 5` | `N = 5` (convention) | `const N = 5` ; `iota` | `const N: usize = 5;` | type obligatoire, pas de constante non typée qui s'adapte. Pas d'`iota` : un `enum` (chap. 6), et `Etat::Up as i32` si tu veux le nombre. |
| Entier | `int`, débordement signé indéfini | illimité | `int` (64 bits), débordement silencieux | `i32` par défaut, `i64`, `u8`…`u128`, `usize` pour les index | **débordement = panic en debug**, modulo en release. Choisis : `n.checked_add(1)`, `wrapping_add`, `saturating_add`. `i32 + i64` ne compile pas, comme en Go. |
| Conversion | implicite | `int(x)` | `float64(n)`, `int(f)` | `n as f64`, `i64::from(n)`, `u8::try_from(n)?` | `as` ne vérifie rien : `300_i32 as u8` vaut 44. `From` pour les conversions sans perte, `TryFrom` pour celles qui peuvent échouer. |
| Booléen | `0` / non-0 | truthy | `true` / `false` | `true` / `false` | comme en Go : `if n {` avec un entier ne compile pas. |
| Null | `NULL` | `None` | `nil` (pointeur, slice, map, interface…) | **n'existe pas** : `Option<T>`, `Some(v)` ou `None` | une référence n'est jamais nulle. Ce qui peut manquer a le type `Option<T>`, et le compilateur t'oblige à traiter `None` avant d'y toucher : fini `nil pointer dereference` (chap. 6). |
| Chaîne | `char*` terminé par `\0` | `str` | `string` immuable | `String` (possédée) et `&str` (empruntée) | deux types là où Go en a un. Paramètre en `&str`, champ et retour en `String`. `s[0]` ne compile pas (`error[E0277]: the type str cannot be indexed by {integer}`) : `s.chars().next()` ou `s.as_bytes()[0]`. |
| Caractère | `'a'` (un `int`) | `"a"` | `'a'` (un `rune`) | `'a'` (un `char`, 4 octets) | `char` n'est pas un octet : `b'a'` est un `u8`. `s.len()` compte les octets, comme `len(s)` en Go. |
| Nommage, visibilité | `static` pour cacher | `_prive` (convention) | **majuscule initiale = exporté** | `pub` devant ; `snake_case` pour les fonctions, `CamelCase` pour les types | la casse ne change rien à la visibilité : sans `pub`, tout est privé au **module**, champs compris. `fn Sonder` donne un avertissement de style, pas un export. |
| Commentaire, doc | `/* */`, `//` | `#`, docstring | `// Sonder vérifie…` au-dessus du symbole | `///` documente ce qui suit, `//!` le module | la doc est du Markdown et ses blocs de code sont des tests (`cargo test --doc`) : un exemple faux casse le build. |

## Opérateurs et contrôle

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Division | `7 / 2 == 3` | `7 / 2 == 3.5`, `7 // 2 == 3` | `7 / 2 == 3`, `7.0 / 2 == 3.5` | `7 / 2 == 3`, `7.0 / 2.0 == 3.5` | `7.0 / 2` ne compile pas (`cannot divide {float} by {integer}`) : aucun mélange, même avec un littéral, là où Go convertit la constante. |
| Modulo | signe du dividende | signe du diviseur | signe du dividende | signe du dividende : `-7 % 2 == -1` ; `(-7_i32).rem_euclid(2) == 1` | sans parenthèses, `-7_i32.rem_euclid(2)` vaut `-1` : l'appel de méthode passe avant le moins unaire. |
| Incrément | `i++` | `i += 1` | `i++` (instruction) | `i += 1` | `++` et `--` n'existent pas du tout. |
| Ternaire | `c ? a : b` | `a if c else b` | n'existe pas | `if c { a } else { b }` | `if` est une **expression** qui renvoie une valeur ; les deux branches ont le même type. |
| Blocs et `;` | `{ }` | indentation | `{ }` obligatoires | `{ }` obligatoires, `;` en fin d'instruction | le `;` compte : `{ x + 1 }` vaut `x + 1`, `{ x + 1; }` vaut `()`. L'erreur typique : `error[E0308]: mismatched types`, `expected i32, found ()`. |
| `if` avec instruction | | | `if err := f(); err != nil {` | `if let Err(e) = f() {` ; `if let Some(x) = o && x > 0 {` | pas d'instruction courte avant la condition : un `let` sur la ligne d'avant, ou `if let` (avec `&&` depuis l'édition 2024). |
| `switch` | `switch`, `case`, `break` | `match` | `switch x { case 1, 2: … default: }` | `match x { 1 \| 2 => …, _ => … }` | **exhaustif** : un cas oublié ne compile pas (`error[E0004]: non-exhaustive patterns`), `_` ramasse le reste. C'est une expression. Pas de `fallthrough`. |
| Boucle compteur | `for (i = 0; i < n; i++)` | `for i in range(n):` | `for i := range n {` | `for i in 0..n {` | `0..n` exclut `n`, `0..=n` l'inclut. Pas de `for` à trois clauses. |
| Sur une collection | index à la main | `for x in xs:` | `for i, x := range xs {` | `for x in &xs {` ; `for (i, x) in xs.iter().enumerate() {` | `for x in xs` **consomme** `xs` : plus utilisable ensuite (E0382). `&xs` emprunte, `&mut xs` permet d'écrire `*x += 1`. |
| `while`, infinie | `while (c)`, `for (;;)` | `while c:` | `for c {`, `for {` | `while c {`, `loop {` | `loop` renvoie une valeur : `let r = loop { break 42; };`. |
| Sortie imbriquée | `goto` | un drapeau | `break Externe` | `break 'externe;` | l'étiquette commence par une apostrophe, comme une durée de vie. |

## Fonctions et erreurs

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Définir | `int f(int a, int b) { }` | `def f(a, b):` | `func f(a, b int) int { }` | `fn f(a: i32, b: i32) -> i32 { a + b }` | chaque paramètre a son type, pas de `a, b int`. La valeur rendue est la dernière expression, sans `return` ni `;`. |
| Retour multiple | struct de sortie | tuple | `func f() (int, error)` | `fn f() -> (i32, String)` ; pour une erreur, `Result<i32, E>` | ne traduis pas `(T, error)` en tuple : c'est un `Result<T, E>` (chap. 8). |
| Arguments par défaut | non | `def f(a, b=2)` | non | non | une struct d'options avec `Default`, ou un builder : `Client::builder().timeout(d).build()`. |
| Nombre variable | `va_list` | `*args` | `func f(xs ...int)` | une tranche : `fn f(xs: &[i32])`, appelée `f(&[1, 2, 3])` | les macros (`println!`, `vec!`) acceptent un nombre variable d'arguments, les fonctions jamais. |
| Passage | par valeur | référence d'objet | par valeur ; `*T` pour modifier | **déplacement** par défaut ; `&T` pour lire, `&mut T` pour modifier | passer une `String` à une fonction la **donne** : l'appelant ne peut plus s'en servir (`error[E0382]: borrow of moved value`). Ce n'est pas la copie de Go (chap. 4). |
| Fonction anonyme | pointeur de fonction | `lambda` | `func(x int) int { return x * 2 }` | `\|x: i32\| x * 2` | capture par emprunt ; `move \|\| …` pour prendre la possession (obligatoire pour un thread). Trois traits selon l'usage : `Fn`, `FnMut`, `FnOnce`. |
| Signaler une erreur | code retour, `errno` | `raise` | `return 0, err` | `return Err(e);` dans une `fn … -> Result<T, E>` | `Result` est un **enum** : impossible de lire la valeur sans passer par l'erreur. Ignorer un `Result` donne `warning: unused Result that must be used`. |
| Propager | `if (r < 0) return r;` | implicite | `if err != nil { return err }` | `let v = f()?;` | `?` fait le `if err != nil` en un caractère et convertit l'erreur par `From`. Seulement dans une fonction qui renvoie `Result` ou `Option`, sinon `error[E0277]: the ? operator can only be used in a function that returns Result or Option`. |
| Ajouter du contexte | | `raise X from e` | `fmt.Errorf("lire %s : %w", p, err)` | `.with_context(\|\| format!("lire {p}"))?` (anyhow) ; `.map_err(…)` (std) | `{e:#}` affiche toute la chaîne de causes, comme un message construit à coups de `%w`. |
| Tester le type d'erreur | comparer `errno` | `except X:` | `errors.Is(err, ErrX)`, `errors.As(err, &cible)` | `match e { ErreurConfig::Manquante(c) => … }` ; `e.downcast_ref::<io::Error>()` (anyhow) | avec `thiserror`, ton erreur est un enum : on fait un `match`, pas de sentinelles ni d'`errors.Is`. |
| Plantage | segfault | traceback | `panic` ; `recover()` dans un `defer` | `panic!` ; `std::panic::catch_unwind` | `catch_unwind` n'est pas un `recover` à semer partout : il ne rattrape rien avec `panic = "abort"`. Un panic dans un thread revient comme `Err` dans `join()`. |
| Nettoyage | à la main | `with`, `finally` | `defer f.Close()` | **rien à écrire** : `Drop` à la fin de portée | pas de `defer` : le fichier se ferme quand sa variable meurt, en ordre inverse de création. `drop(f)` pour libérer plus tôt. |
| `main` | `int main(int argc, char **argv)` | `if __name__ == "__main__":` | `func main()` + `os.Exit(1)` | `fn main() -> anyhow::Result<()>` | une erreur renvoyée par `main` s'affiche (`Error: …`) et le code de sortie vaut 1. `std::process::exit` saute les `Drop`, comme `os.Exit` saute les `defer`. |

## Collections

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Tableau fixe | `int t[10]` | (liste) | `[10]int` | `[i32; 10]`, `[0; 10]` | la taille fait partie du type, comme en Go. |
| Tableau dynamique | `malloc` + `realloc` | `list` | `[]int`, `s = append(s, x)` | `Vec<i32>`, `v.push(x)` | `push` modifie en place : pas de réaffectation, mais il faut `let mut v`. |
| Tranche | pointeur + taille | `xs[1:3]` (copie) | `s[1:3]` (partage) | `&v[1..3]`, de type `&[i32]` | une tranche est un **emprunt** : tant qu'elle vit, `v.push(…)` est refusé (`error[E0502]: cannot borrow v as mutable because it is also borrowed as immutable`). Le cas où Go réalloue en silence et laisse la vieille tranche sur l'ancien tableau ne compile tout simplement pas. |
| Slice nil | `NULL` | `None` | `var s []int` | `Vec::new()` | pas d'état nil : un `Vec` vide n'alloue rien. `Option<Vec<T>>` si tu dois distinguer « absent » et « vide ». |
| Hors bornes | comportement indéfini | `IndexError` | panic | panic ; `v.get(i)` renvoie `Option<&T>` | `get` est la version sans panic. |
| Dictionnaire | à écrire | `dict` | `map[string]int`, `v, ok := m[k]` | `HashMap<String, i32>` (`use std::collections::HashMap`), `m.get(k)` | `m["x"]` **panique** si la clé manque (Go rend zéro) : `m.get("x")` renvoie un `Option`. `*m.entry(k).or_insert(0) += 1` remplace `m[k]++`. |
| Ordre d'itération | | insertion | aléatoire | aléatoire ; `BTreeMap` pour des clés triées | `BTreeMap` donne l'ordre trié sans `slices.Sorted(maps.Keys(m))`. |
| Ensemble | à écrire | `set` | `map[string]struct{}` | `HashSet<String>` | un vrai type, avec `union`, `intersection`, `difference`. |
| Copier | `memcpy` | `copy.deepcopy` | `slices.Clone(s)` | `v.clone()` | `let b = a;` **déplace** le `Vec` (un seul propriétaire), il ne le partage pas comme une slice Go. |
| Trier | `qsort` | `sorted(xs)` | `slices.Sort(s)`, `slices.SortFunc` | `v.sort()`, `v.sort_by_key(\|h\| h.port)`, `v.sort_by(\|a, b\| a.cmp(b))` | `f64` n'est pas `Ord` (à cause de NaN) : `v.sort_by(f64::total_cmp)`. |
| Transformer | boucle | compréhension | boucle `for` | `v.iter().map(f).filter(p).collect::<Vec<_>>()` | paresseux : sans `collect` ni `for`, rien ne s'exécute (`warning: unused Map that must be used`). |
| Tuple | non | `(1, "a")` | n'existe pas | `(1, "a")`, champ `t.0` | un vrai type, utilisable comme clé de `HashMap`. |
| Chaîne de format | `printf("%d %s")` | `f"{n} {s}"` | `fmt.Sprintf("%d %v", n, v)` | `format!("{n} {v:?}")` | `{}` exige `Display`, `{:?}` exige `Debug` (`#[derive(Debug)]`, sinon `error[E0277]: P doesn't implement Debug`). Vérifié par le compilateur, pas par `go vet`. |

## Structures, méthodes, interfaces

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Structure | `struct` | `@dataclass` | `type Hote struct { Nom string }` | `struct Hote { nom: String }` | pas de valeur zéro : tous les champs dans le littéral, ou `..Default::default()`. |
| Constructeur | fonction | `__init__` | `func NouvelHote(nom string) *Hote` | `impl Hote { fn new(nom: &str) -> Self { … } }` | `new` n'est qu'une convention : une fonction associée, appelée `Hote::new("a")`. |
| Méthode | fonction + pointeur | `def m(self):` | `func (h *Hote) M()` | `fn m(&self)`, `fn m(&mut self)`, `fn m(self)` dans un `impl` | trois receveurs : lire, modifier, consommer. `&mut self` joue le rôle du receveur pointeur ; pas de règle « tout en pointeur si un l'est ». |
| Énumération | `enum` | `Enum` | `const ( A = iota; B )` | `enum Etat { Up, Down { depuis: u64 } }` | une variante porte ses propres données. Impossible de fabriquer un `Etat(42)` hors liste. |
| Interface | pointeurs de fonction | `Protocol` | `type Sondeur interface { Sonder() error }`, satisfaite implicitement | `trait Sonde { fn sonder(&self) -> bool; }` puis `impl Sonde for Tcp { … }` | **explicite** : avoir la méthode ne suffit pas, il faut écrire `impl Sonde for Tcp`. En échange, tu peux implémenter ton trait sur un type qui n'est pas à toi (`impl Sonde for String`). |
| Polymorphisme | `void*` | duck typing | une valeur d'interface | `fn f(s: &impl Sonde)` (statique) ; `&dyn Sonde`, `Box<dyn Sonde>` (dynamique) | générique par défaut : une copie compilée par type, appel direct. `dyn` quand des types différents cohabitent dans un `Vec`. |
| Assertion de type | cast | `isinstance` | `v, ok := s.(*Tcp)` ; `switch v := x.(type) {` | un `match` sur un enum ; `Any::downcast_ref` en dernier recours | on ne demande pas le type caché : on modélise les cas avec un `enum`. |
| Embarquer | struct dans struct | héritage | `type Serveur struct { Hote }` promeut champs et méthodes | composition : `struct Serveur { hote: Hote }`, puis `s.hote.nom` | aucune promotion de champs ni de méthodes. Pour partager un comportement : une méthode par défaut dans un trait. |
| `String()`, `Error()` | | `__str__` | `fmt.Stringer`, `error` | `impl fmt::Display`, `impl std::error::Error` | `Display` donne `{}` et `.to_string()` d'un coup. |
| Générique | macros, `void*` | duck typing | `func Max[T cmp.Ordered](a, b T) T` | `fn max<T: PartialOrd>(a: T, b: T) -> T` | partout dans la std (`Vec<T>`, `Option<T>`, `Result<T, E>`), pas seulement en dernier recours comme en Go. |
| Égalité | `memcmp` | `__eq__` | `==` si tous les champs sont comparables | `#[derive(PartialEq)]` puis `==` | rien n'est comparable par défaut, il faut le dériver ; `Eq` et `Hash` en plus pour une clé de `HashMap`. |

## Mémoire et pointeurs

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Allocation | `malloc` / `free` | automatique | automatique (GC) | **possession** : libéré quand le propriétaire sort de portée | ni GC ni `free` : la libération est déterministe, au `}` (chap. 4). |
| Affectation | copie | copie l'étiquette | copie la valeur | **déplace** (sauf les types `Copy` : entiers, `f64`, `bool`, `char`, `&T`) | `let b = a;` puis `a` : E0382. `a.clone()` quand tu veux vraiment deux exemplaires. |
| Pointeur | `int *p = &n; *p = 3;` | non | `p := &n; *p = 3` | `let r = &mut n; *r = 3;` | une référence a une durée de vie : renvoyer `&local` ne compile pas (`error[E0515]: cannot return reference to local variable`). En Go c'est valide (le GC fait vivre la variable) ; ici, renvoie la valeur elle-même. |
| Sur le tas | `malloc` | tout | `&Hote{}` qui s'échappe | `Box::new(h)` | explicite, et rare en dehors des types récursifs et des `Box<dyn Trait>` (chap. 10). |
| Partager | pointeur | tout est partagé | n'importe quel pointeur | `Rc<T>` (un thread), `Arc<T>` (plusieurs) | partager se paie et se voit dans le type. |
| Modifier ce qui est partagé | pointeur | naturel | `p.Champ = x` | `Rc<RefCell<T>>`, `Arc<Mutex<T>>` | partage **et** modification demandent une cellule : c'est la règle « plusieurs `&` ou un seul `&mut` ». |
| Pointeur nul | segfault | `AttributeError` | panic | n'existe pas en Rust sûr | `Option<Box<T>>` a la taille d'un simple pointeur : `None` est codé par l'adresse 0. |
| Destructeur | `free` | `__del__`, `with` | `defer`, `runtime.AddCleanup` | `impl Drop for Garde { fn drop(&mut self) { … } }` | appelé à coup sûr en fin de portée : fichiers, verrous, connexions se ferment seuls. |
| Taille en mémoire | `sizeof` | `sys.getsizeof` | `unsafe.Sizeof(x)` | `std::mem::size_of::<T>()` | l'ordre des champs n'est pas garanti (le compilateur réordonne) sauf avec `#[repr(C)]`. |
| Code non vérifié | tout | `ctypes` | `unsafe.Pointer` | `unsafe { … }` | les règles d'emprunt restent actives dans `unsafe` : il autorise seulement les pointeurs bruts, la FFI, `static mut` (chap. 10). |

## Concurrence

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Lancer | `pthread_create` | `threading.Thread` | `go f(x)` | `thread::spawn(move \|\| f(x))` ou `tokio::spawn(async move { … })` | deux mondes : un thread de l'OS (cher, quelques centaines au plus) ou une tâche async (légère comme une goroutine, mais seulement dans un runtime tokio, chap. 13). |
| Attendre | `pthread_join` | `t.join()` | `sync.WaitGroup` | `h.join()`, `thread::scope`, `t.await` | le `JoinHandle` rend la valeur de retour du thread : pas besoin d'un channel pour ça. `main` qui se termine tue les threads, comme en Go. |
| Channel | pipe | `queue.Queue` | `ch := make(chan int, 10)` | `let (tx, rx) = mpsc::channel();`, `mpsc::sync_channel(10)` ; `tokio::sync::mpsc::channel(10)` | un seul récepteur (`mpsc` = plusieurs producteurs, un consommateur). Pas de `close` : le channel se ferme quand tous les `tx` sont détruits, d'où le `drop(tx)` avant de lire. |
| `select` | `select(2)` | `asyncio.wait` | `select { case v := <-ch: … }` | `tokio::select! { v = rx.recv() => …, _ = sleep(d) => … }` | async seulement ; les branches perdantes sont **annulées** (leur future est détruite). Rien d'équivalent sur les channels de la std. |
| Mutex | `pthread_mutex_lock` | `threading.Lock` | `mu.Lock(); defer mu.Unlock()` | `let m = Mutex::new(0); *m.lock().unwrap() += 1;` | le Mutex **contient** la donnée : impossible d'y toucher sans verrouiller. Le déverrouillage se fait quand le garde est détruit. |
| Partager entre threads | pointeur global | variable globale | variable capturée | `Arc<Mutex<T>>`, `Arc::clone(&a)` par thread | capturer une variable locale par référence dans `thread::spawn` ne compile pas (`error[E0373]: closure may outlive the current function`) : `move` et `Arc`, ou `thread::scope`. |
| Course de données | comportement indéfini | le GIL cache le problème | `go test -race` à l'exécution | **erreur de compilation** | `Send` et `Sync` vérifient à la compilation ce que `-race` cherche à l'exécution (`error[E0277]: Rc<String> cannot be sent between threads safely`). Les interblocages, eux, restent possibles. |
| Atomique | `stdatomic.h` | non | `var n atomic.Int64; n.Add(1)` | `static N: AtomicU64 = AtomicU64::new(0); N.fetch_add(1, Ordering::Relaxed);` | tu choisis l'ordre mémoire : `Relaxed` pour un compteur, `SeqCst` dans le doute. |
| Annulation, délai | `alarm` | `asyncio.timeout` | `context.WithTimeout(ctx, d)` | `tokio::time::timeout(d, fut).await` ; abandonner une future l'annule | pas de `ctx` passé à toutes les fonctions : détruire la future suffit. Pour annuler à distance : `tokio::sync::watch`, ou `CancellationToken` de la crate `tokio-util`. |
| Une seule fois | `pthread_once` | variable de module | `sync.Once`, `sync.OnceValue` | `static C: LazyLock<T> = LazyLock::new(\|\| …);`, `OnceLock` | initialisé au premier accès, sûr entre threads. |

## Entrées-sorties, fichiers, JSON, HTTP

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Lire un fichier | `fopen`, `fread` | `open(p).read()` | `os.ReadFile(p)` | `fs::read_to_string(p)?` | renvoie une `String` (UTF-8 vérifié, erreur sinon) ; `fs::read` pour des octets. |
| Ligne par ligne | `fgets` | `for l in f:` | `bufio.NewScanner(f)` | `for l in BufReader::new(f).lines() { let l = l?; }` | chaque ligne est un `Result`. Sans `BufReader`, chaque lecture est un appel système. Pas de limite de 64 Ko par ligne comme avec `Scanner`. |
| Écrire | `fprintf` | `f.write(s)` | `os.WriteFile(p, b, 0o644)` | `fs::write(p, contenu)?`, `writeln!(f, "ok")?` | `writeln!` sur un `File` demande `use std::io::Write`, sinon `error[E0599]: cannot write into File`. |
| Chemins | `char*` | `pathlib.Path` | `filepath.Join(a, b)` | `Path::new("a").join("b")`, `PathBuf` | `Path` et `PathBuf` sont la paire `&str` et `String` des chemins ; pas forcément UTF-8, `p.display()` pour afficher. |
| Environnement, arguments | `getenv`, `argv` | `os.environ`, `sys.argv` | `os.Getenv("DSN")`, `os.Args` | `env::var("DSN")` (un `Result`), `env::args()` | `var` distingue « absente » et « vide » ; `env::set_var` est `unsafe` en édition 2024. |
| Options de ligne de commande | `getopt` | `argparse` | `flag.String("addr", ":8080", "adresse")` | `#[derive(clap::Parser)] struct Args { … }` | la struct est la documentation : `--help` offert, types vérifiés au parsing. |
| Sous-processus | `fork` + `exec` | `subprocess.run` | `exec.Command("ls", "-l").Output()` | `Command::new("ls").arg("-l").output()?` | `output()` réussit même si la commande sort en 1 : teste `sortie.status.success()`. |
| JSON | à la main | `json.loads(s)` | `json.Unmarshal(data, &v)` et des tags | `serde_json::from_str::<T>(&s)?` avec `#[derive(Deserialize)]` | pas de champs exportés à prévoir : `#[serde(rename = "…")]` si les noms diffèrent. Un champ absent est une **erreur** (Go met la valeur zéro), sauf en `Option<T>` ou avec `#[serde(default)]`. |
| Client HTTP | libcurl | `requests.get(url)` | `client := &http.Client{Timeout: 5 * time.Second}` | `reqwest::Client::builder().timeout(d).build()?` | async (`.await`). Un 404 n'est pas une erreur : `.error_for_status()?`. |
| Serveur HTTP | à la main | FastAPI | `mux.HandleFunc("GET /taches/{id}", h)` | `Router::new().route("/taches/{id}", get(h))` (axum) | les extracteurs (`Path`, `Json`, `State`) sont des arguments typés du handler, et le corps (`Json`) vient en dernier. |
| SQL | libpq | `psycopg` | `db.QueryRowContext(ctx, q, nom).Scan(&ip)` | `conn.query_row(q, [nom], \|l\| l.get(0))?` (rusqlite) ; `client.query(q, &[&nom]).await?` (postgres) | pas d'interface commune comme `database/sql` : chaque crate a son API. `l.get::<_, i32>(0)` sur une colonne `BIGINT` panique avec tokio-postgres ; `try_get` renvoie un `Err` à la place. |
| Temps | `time()` | `time.monotonic()` | `time.Now()`, `time.Since(t)`, `5 * time.Second` | `Instant::now()`, `t.elapsed()`, `Duration::from_secs(5)` | pas de dates calendaires dans la std : `chrono` ou `jiff`. |
| Journal | `fprintf(stderr)` | `logging` | `log/slog` | `eprintln!`, puis `tracing` | pas de journal structuré dans la std. |

## Outillage et build

| Quoi | C | Python | Go | Rust | Piège |
|---|---|---|---|---|---|
| Chaîne d'outils | `cc`, `ld`, `make` | interpréteur | `go build` | `cargo build` | la compilation est plus lente qu'en Go : `cargo check` pour la boucle courte. |
| Projet, dépendances | `Makefile`, `-lfoo` | `pyproject.toml`, venv | `go.mod`, `go.sum` | `Cargo.toml`, `Cargo.lock` | une dépendance est un nom de crate sur crates.io, pas une URL ; `cargo add`. Les features choisissent des morceaux d'une crate : `features = ["derive"]`. |
| Unité de code | fichier `.c` | module `.py` | dossier = paquet | crate = unité compilée ; module = fichier ou bloc `mod` | un fichier n'existe pour le compilateur que s'il est déclaré par `mod nom;` dans son parent : créer `reseau.rs` ne suffit pas, à l'inverse de Go. |
| Import | `#include` | `import x` | `import "module/paquet"` | `use crate::reseau::Cible;` | un `use` inutilisé n'est qu'un avertissement. Deux modules d'une même crate peuvent se citer mutuellement. |
| Plusieurs paquets | | monorepo | `go work` | `[workspace]` dans le `Cargo.toml` racine | un seul `target/` et un seul `Cargo.lock` pour tous. |
| Tests | cmocka | `pytest` | `go test ./...`, `func TestX(t *testing.T)` | `cargo test`, `#[test] fn x() { assert_eq!(a, b); }` | les tests unitaires vivent dans le même fichier (`#[cfg(test)] mod tests`), ceux d'intégration dans `tests/`. Ils tournent en parallèle par défaut. `cargo test nom` filtre comme `-run`. |
| Benchmark | à la main | `timeit` | `func BenchmarkX(b *testing.B)` | une crate : `criterion` ou `divan` | l'attribut `#[bench]` de la std n'existe qu'en nightly. |
| Détecteur de courses | ThreadSanitizer | non | `go test -race` | inutile en Rust sûr | le compilateur refuse la course avant l'exécution. |
| Formatage | au choix | `ruff format` | `gofmt` | `cargo fmt` | quatre espaces, pas de tabulations. `cargo fmt --check` en CI. |
| Lint | `-Wall` | `ruff` | `go vet`, `golangci-lint` | `cargo clippy` | livré avec rustup ; `cargo clippy --all-targets -- -D warnings` en CI. |
| Compilation croisée | une chaîne par cible | non | `GOOS=linux GOARCH=arm64 go build` | `rustup target add aarch64-unknown-linux-gnu` puis `cargo build --target aarch64-unknown-linux-gnu` | la bibliothèque standard de la cible se télécharge, mais il faut souvent un éditeur de liens pour elle : `cross` (via Docker) ou `cargo zigbuild` s'en chargent (chap. 15). |
| Binaire statique | `-static` | non | `CGO_ENABLED=0` | `--target x86_64-unknown-linux-musl` | sur Linux, le binaire par défaut dépend de la glibc : musl pour tourner dans `FROM scratch`. |
| Debug et release | `-O0`, `-O2` | non | un seul mode | `cargo build` (debug) et `cargo build --release` | un binaire debug est souvent dix fois plus lent, ou pire : ne mesure jamais une performance sans `--release`. |
| Version du langage | C99, C11 | 3.x | `go 1.27` dans `go.mod` | `edition = "2024"` et `rust-version` dans `Cargo.toml` | l'édition change la syntaxe crate par crate, et des crates d'éditions différentes se mélangent sans souci. |
| Documentation | man | `help()` | `go doc`, pkg.go.dev | `cargo doc --open`, docs.rs | docs.rs héberge la doc de chaque version de chaque crate publiée. |
| Comprendre une erreur | | | | `rustc --explain E0502` | chaque code d'erreur a sa page, avec un exemple et la correction. |

## Le réflexe à prendre

En C, tu penses en octets et en adresses ; en Python, en objets et en étiquettes ; en Go, en valeurs qui se copient et en petits en-têtes qui partagent. En Rust, pense en **propriétaires** : chaque valeur en a exactement un, qui la libère en sortant de portée ; tout le reste l'emprunte, soit à plusieurs en lecture, soit à un seul en écriture, et jamais plus longtemps qu'elle ne vit. Le reste en découle : le déplacement à l'affectation, `Option` à la place de `nil`, `Result` à la place de `error`, `Drop` à la place de `defer`, `Arc<Mutex<T>>` à la place d'une variable partagée par des goroutines. Quand le compilateur refuse, il ne chipote pas : il te montre l'endroit où, en Go, tu aurais eu un bug à l'exécution. C'est le chapitre 4 pour la possession, le 8 pour les erreurs, le 11 pour la concurrence.
