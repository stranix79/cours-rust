# Labo 10 : Sous le capot

**Chapitre** : [10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe](../../cours/10-sous-le-capot.md)

## Objectif
Cinq petites expériences, une par module, comme le labo Go du même nom : chacune isole un mécanisme du chapitre 10 et le rend visible par un test. Un compteur partagé en `Rc<RefCell<T>>` dont on regarde le nombre de propriétaires monter et descendre, une liste chaînée en `Box` qu'on libère sans faire exploser la pile, un arbre avec des `Weak` vers le parent dont on prouve qu'il ne fuit pas, deux appels à la libc par FFI enfermés dans des fonctions sûres, et la taille réelle des types que tu manipules depuis le chapitre 2. À la fin, `Rc::strong_count`, `try_borrow_mut`, `Option::take`, `Rc::downgrade`, `unsafe extern "C"` et `size_of` ne sont plus des mots, ce sont des choses que tu as vues.

## Consignes
1. Lis `src/lib.rs` (les cinq `pub mod`) et `src/main.rs` (fourni : il fait tourner chaque expérience et affiche). Lance `cargo test` : ça compile, et ça échoue. `cargo test --no-fail-fast` montre tous les échecs, binaire par binaire.
2. `src/compteur.rs` : `Capteur::new` (TODO 1) clone le `Rc`, pas le compteur ; `mesurer` (TODO 2) écrit à travers `&self` avec `borrow_mut()` ; `emprunt_pendant_lecture` (TODO 3) garde un `borrow()` vivant et tente `try_borrow_mut()`, qui rend une `Err` au lieu de paniquer. Remplace ensuite `try_borrow_mut()` par `borrow_mut()`, relance et lis le message de la panique (`RefCell already borrowed`), puis remets `try_`. `cargo test compteur`.
3. `src/liste.rs` : `push`, `pop`, `peek` (TODO 4a, 4b, 4c) avec `Option::take`, `map` et `as_ref`, puis `next` de l'itérateur (TODO 5) avec `as_deref`. Lance `cargo test liste` **avant** le TODO 6 : le binaire de tests s'arrête net sur `has overflowed its stack`. C'est le `Drop` récursif par défaut sur un million de maillons. Écris le `Drop` itératif (TODO 6) et relance.
4. `src/arbre.rs` : `ajouter_enfant` (TODO 7), un `Weak` vers le haut et un `Rc` vers le bas, puis `chemin` (TODO 8), qui remonte avec `upgrade()`. Le test `arbre_sans_fuite` compte les `Drop` avec un `AtomicUsize` statique : remplace pour voir le `Weak` du parent par un `Rc` (dans le type et dans `nouveau`), relance, et regarde le test échouer avec 0 nœud détruit sur 7. Remets le `Weak`. `cargo test arbre`.
5. `src/ffi.rs` : déclare `strlen` et `getpid` dans un bloc `unsafe extern "C"` (TODO 9a), puis écris les enveloppes sûres `longueur_c` et `pid` (TODO 9b, 9c), chacune avec son commentaire `// SAFETY :`. `cargo test ffi`. Note que `strlen(c"été")` vaut 5 : la libc compte des octets.
6. `src/tailles.rs` : complète la liste (TODO 10). **Avant** de lancer, écris sur un papier la taille que tu attends pour chaque type, puis `cargo run` et compare. Les tests vérifient la niche (`Option<Box<u8>>` fait la taille d'un `Box<u8>`) et les gros pointeurs (`&str`, `Box<dyn Fn()>`). `cargo test tailles`.
7. `cargo test` : tout vert. `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` : rien à redire. Retire au passage les `#[allow(dead_code)]` qui ne servent plus.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test --no-fail-fast        # tous les binaires de test, même après un échec
cargo test liste                 # les tests dont le nom contient « liste »
cargo run                        # les cinq expériences
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run -q
== Compteur partagé (Rc<RefCell<T>>)
propriétaires au départ : 1
avec deux capteurs      : 3
cuisine et salon ont écrit dans le même compteur
après leur destruction  : 1
compteur : Compteur { total: 62, mesures: 3 }
emprunt mutable pendant une lecture : refusé (RefCell already borrowed)

== Liste chaînée (Option<Box<Noeud<T>>>)
contenu : ["trois", "deux", "un"], len = 3, tête = Some("trois")
pop : Some("trois"), reste 2
un million de maillons : len = 1000000
un million de maillons libérés sans exploser la pile

== Arbre (Rc vers le bas, Weak vers le haut)
chemin : racine/etc/nginx
etc : 2 fort(s), 1 faible(s)
nœuds détruits en sortant du bloc : 3

== FFI (libc)
strlen("bonjour") = 7
strlen("été")     = 5 (des octets, pas des lettres)
getpid() = 4713, std::process::id() = 4713

== Tailles (size_of, en octets)
u8                    1
u64                   8
u128                 16
char                  4
bool                  1
()                    0
&u8                   8
&str                 16
&[u64]               16
String               24
Vec<u8>              24
Box<u8>               8
Rc<u8>                8
Option<u8>            2
Option<u32>           8
Option<Box<u8>>       8
Option<String>       24
Option<char>          4
Box<dyn Fn()>        16
[u8; 100]           100
```
Le pid change à chaque lancement. Les tailles sont celles d'une machine 64 bits (Apple Silicon ou x86_64) ; sur une cible 32 bits, tout ce qui contient un pointeur est divisé par deux.

## Pour aller plus loin
- Installe Miri (`rustup +nightly component add miri`) et lance `cargo +nightly miri test liste` dans la solution : Miri exécute les tests dans un interpréteur qui vérifie chaque accès mémoire. Puis écris dans un test un `unsafe { *std::ptr::null::<u8>() }` et regarde Miri le dénoncer avec `Undefined Behavior`, là où un lancement normal plante sans explication (ou pire, ne plante pas).
- Ajoute à la liste un `IntoIter` (le `for x in liste` qui consomme) et un `iter_mut()`, puis une méthode `inverser(&mut self)` en O(n) sans allocation. C'est l'exercice de « Learning Rust With Entirely Too Many Linked Lists », le meilleur texte sur ce que le vérificateur d'emprunts accepte et pourquoi.
- Remplace `Rc` par `Arc` et `RefCell` par `Mutex` dans `compteur.rs`, puis fais mesurer les capteurs depuis plusieurs threads : c'est exactement le passage du chapitre 11. Essaie d'abord avec `Rc` et lis l'erreur `E0277` : `Rc<RefCell<Compteur>>` cannot be sent between threads safely.
