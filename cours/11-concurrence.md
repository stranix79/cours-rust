# 11. Concurrence : threads, channels, Arc et Mutex, Send et Sync

*Rust de zéro à la prod : chapitre 11 sur 16.* ← [10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe](10-sous-le-capot.md) · [Sommaire](../README.md) · [12. Fichiers, JSON, ligne de commande DevOps](12-fichiers-json-cli.md) →

Ton script de supervision sonde cinquante serveurs à la suite, deux secondes de délai chacun : trois en panne, et le tour prend six secondes de plus. En Go, tu aurais lancé une goroutine par cible. En Rust, la bibliothèque standard te donne de vrais threads système, des canaux pour faire circuler les résultats, des verrous qui **contiennent** la donnée qu'ils protègent, et surtout deux traits, `Send` et `Sync`, grâce auxquels le compilateur refuse tout programme qui contiendrait une course de données. Ce n'est pas un détecteur qu'on lance en CI comme le `-race` de Go : c'est une erreur de compilation. Ce chapitre montre les outils, puis ce que cette garantie couvre, et ce qu'elle ne couvre pas.

### 11.1 Lancer et attendre : `thread::spawn` et `join`

`thread::spawn(f)` lance la fermeture `f` dans un nouveau thread du système et rend tout de suite un `JoinHandle`. `join()` attend la fin du thread et rend **ce que la fermeture a renvoyé**, dans un `Result` qui vaut `Err` si le thread a paniqué. Là où une goroutine ne rend rien, un thread Rust rend une valeur.

```rust
use std::thread;
use std::time::{Duration, Instant};

fn sonde(nom: &str, d: Duration) -> String {
    thread::sleep(d);
    format!("{nom} répond après {d:?}")
}

fn main() {
    let debut = Instant::now();
    let mut poignees = Vec::new();
    for nom in ["db1", "db2", "cache"] {
        // spawn rend un JoinHandle : la poignée pour attendre le thread et
        // récupérer ce que sa fermeture renvoie.
        poignees.push(thread::spawn(move || sonde(nom, Duration::from_millis(100))));
    }
    for p in poignees {
        // join attend la fin et rend Result : Err si le thread a paniqué.
        println!("{}", p.join().unwrap());
    }
    println!("trois sondes en {} ms", debut.elapsed().as_millis());
}
```

```
$ cargo run -q
db1 répond après 100ms
db2 répond après 100ms
cache répond après 100ms
trois sondes en 101 ms
```

Trois attentes de 100 ms en 101 ms. Les résultats sortent dans l'ordre des poignées, pas dans l'ordre de fin : on attend `db1`, puis `db2`, puis `cache`. Et comme en Go, si `main` se termine, le processus se termine, threads compris : un thread qu'on n'a pas `join` est tué sans prévenir.

Le mot `move` n'est pas décoratif. Sans lui, la fermeture **emprunterait** les variables qu'elle utilise, et le thread pourrait leur survivre :

```rust
use std::thread;

fn main() {
    let hotes = vec!["db1".to_string(), "db2".to_string()];
    let t = thread::spawn(|| {
        println!("{} hôtes", hotes.len());
    });
    t.join().unwrap();
}
```

```
$ cargo build
error[E0373]: closure may outlive the current function, but it borrows `hotes`, which is owned by the current function
 --> src/bin/c11_move.rs:5:27
  |
5 |     let t = thread::spawn(|| {
  |                           ^^ may outlive borrowed value `hotes`
6 |         println!("{} hôtes", hotes.len());
  |                              ----- `hotes` is borrowed here
  |
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `hotes` (and any other referenced variables), use the `move` keyword
  |
5 |     let t = thread::spawn(move || {
  |                           ++++
```

`spawn` exige une fermeture `'static` : qui ne contient aucune référence de durée limitée (chapitre 10), parce que rien ne garantit que le thread finira avant la fonction qui l'a lancé. Ici le `join()` est juste en dessous, mais le compilateur ne raisonne pas sur ce que tu feras plus tard : il raisonne sur le type de `spawn`. `move` déplace `hotes` dans la fermeture, le thread en devient propriétaire, et `main` ne peut plus s'en servir. Pour que deux threads lisent la même donnée, il faut soit la partager (`Arc`, 11.3), soit garantir que les threads finissent avant elle (`scope`, 11.2).

**Venant du C :** `spawn` est `pthread_create` et `join` est `pthread_join`, avec la valeur de retour typée au lieu d'un `void *` à caster, et la panique d'un thread rapportée proprement au lieu d'un processus qui meurt.

**Venant de Go :** une goroutine pèse 2 Ko de pile au départ et l'exécutif Go en multiplexe des centaines de milliers sur quelques threads. Un thread Rust est un **vrai thread du système** : 2 Mio de pile réservés par défaut (`thread::Builder::new().stack_size(...)` pour changer), un appel système pour le créer, l'ordonnanceur du noyau pour le faire tourner. Dix mille threads, ça passe ; cent mille, non. Pour sonder cinquante serveurs, les threads sont parfaits ; pour tenir cent mille connexions ouvertes, c'est l'async du chapitre 13. Et `sync.WaitGroup` n'a pas d'équivalent nécessaire : on garde les `JoinHandle` dans un `Vec` et on les `join`, ou on utilise `thread::scope`, qui attend tout le monde tout seul.

### 11.2 `thread::scope` : des threads qui empruntent

Depuis Rust 1.63, `thread::scope` lance des threads qui ont le droit d'**emprunter** les variables locales, parce que le scope garantit qu'ils sont tous terminés avant de rendre la main :

```rust
use std::thread;

fn main() {
    let journal = vec!["INFO a", "ERROR b", "WARN c", "ERROR d", "INFO e", "ERROR f"];
    let mut total = 0;

    // Les threads d'un scope peuvent EMPRUNTER : le scope attend qu'ils
    // aient tous fini avant de rendre la main, donc `journal` leur survit.
    thread::scope(|s| {
        let moities = journal.chunks(journal.len() / 2);
        let poignees: Vec<_> = moities
            .map(|morceau| s.spawn(move || morceau.iter().filter(|l| l.starts_with("ERROR")).count()))
            .collect();
        for p in poignees {
            total += p.join().unwrap();
        }
    });
    println!("{total} erreurs dans {} lignes", journal.len());
}
```

```
$ cargo run -q
3 erreurs dans 6 lignes
```

Chaque thread reçoit une tranche `&[&str]` du `Vec`, sans copie et sans `Arc`. Le `move` déplace la **référence** `morceau` dans la fermeture, pas les données. C'est la forme à préférer chaque fois que les threads ont une durée de vie bornée par une fonction : découper un travail, sonder une liste de cibles, traiter les fichiers d'un dossier. Le labo 11 l'utilise pour que ses workers empruntent la liste des cibles.

### 11.3 `Send` et `Sync` : le compilateur connaît les threads

Deux traits marqueurs, sans méthode, que le compilateur implémente tout seul pour chaque type selon ce qu'il contient :

- un type est **`Send`** si on peut **transférer** une valeur de ce type à un autre thread ;
- un type est **`Sync`** si on peut **partager** des `&T` entre threads (autrement dit, `T` est `Sync` si `&T` est `Send`).

Presque tout est `Send` et `Sync` : les nombres, `String`, `Vec<T>` si `T` l'est, tes structs si leurs champs le sont. Les exceptions sont précisément les types dangereux entre threads. `Rc<T>` n'est ni l'un ni l'autre : son compteur de références est un entier ordinaire, et deux threads qui feraient `Rc::clone` en même temps pourraient perdre un incrément, puis libérer la valeur alors qu'un thread s'en sert encore. Le compilateur refuse :

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let config = Rc::new(String::from("postgres://db1/app"));
    let copie = Rc::clone(&config);
    let t = thread::spawn(move || println!("{copie}"));
    t.join().unwrap();
}
```

```
$ cargo build
error[E0277]: `Rc<String>` cannot be sent between threads safely
 --> src/bin/c11_rc.rs:7:27
  |
7 |     let t = thread::spawn(move || println!("{copie}"));
  |             ------------- -------^^^^^^^^^^^^^^^^^^^^
  |             |             |
  |             |             `Rc<String>` cannot be sent between threads safely
  |             |             within this `{closure@src/bin/c11_rc.rs:7:27: 7:34}`
  |             required by a bound introduced by this call
  |
  = help: within `{closure@src/bin/c11_rc.rs:7:27: 7:34}`, the trait `Send` is not implemented for `Rc<String>`
note: required by a bound in `spawn`
```

La signature de `thread::spawn` exige `F: Send + 'static` : la fermeture et tout ce qu'elle capture. La correction est **`Arc<T>`** (*atomically reference counted*) : le même `Rc`, avec un compteur atomique. Il est `Send` et `Sync` dès que `T` l'est, coûte un peu plus cher par clone (une instruction atomique au lieu d'une addition), et c'est tout. `Arc::new`, `Arc::clone`, `Arc::strong_count` : même API que `Rc`. De même, `RefCell` n'est pas `Sync` (son compteur d'emprunts n'est pas atomique) ; son équivalent entre threads est le `Mutex`.

Ces deux traits sont toute la magie. Il n'y a pas de règle spéciale pour les threads dans le langage : `spawn`, les canaux et les verrous de la bibliothèque standard ont des bornes `Send` et `Sync` dans leurs signatures, et le système de types fait le reste. Une crate externe (rayon, tokio, crossbeam) a exactement les mêmes garanties, par les mêmes bornes.

### 11.4 `Arc<Mutex<T>>` : un état partagé et modifiable

Un `Mutex<T>` Rust **enveloppe** la donnée qu'il protège. `lock()` rend un `MutexGuard<T>`, par lequel on lit et on écrit, et le verrou est relâché quand le guard est détruit. Il est impossible de toucher à la donnée sans avoir le verrou, et impossible d'oublier de le relâcher.

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Le Mutex ENVELOPPE la donnée : impossible d'y toucher sans le verrou.
    let stats: Arc<Mutex<HashMap<u16, u32>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut poignees = Vec::new();
    for t in 0..4 {
        let stats = Arc::clone(&stats);
        poignees.push(thread::spawn(move || {
            for i in 0..250 {
                let code = 200 + ((t * 250 + i) % 3) as u16 * 100;
                // lock() rend un MutexGuard : le verrou est tenu tant qu'il vit.
                let mut s = stats.lock().unwrap();
                *s.entry(code).or_insert(0) += 1;
            } // le guard est détruit à chaque tour : verrou relâché
        }));
    }
    for p in poignees {
        p.join().unwrap();
    }
    let s = stats.lock().unwrap();
    let mut codes: Vec<_> = s.iter().collect();
    codes.sort();
    println!("{codes:?}, {} référence(s) à l'Arc", Arc::strong_count(&stats));
}
```

```
$ cargo run -q
[(200, 334), (300, 333), (400, 333)], 1 référence(s) à l'Arc
```

Mille incréments répartis sur quatre threads, aucun perdu. L'`Arc` permet à chaque thread d'être propriétaire du même `Mutex` ; le `Mutex` permet de modifier à travers une référence partagée. `Arc<Mutex<T>>` est le `Rc<RefCell<T>>` du chapitre 10, version multi-thread, et c'est le motif que tu écriras le plus souvent.

**La portée du verrou** est celle du guard. Le tenir le moins longtemps possible est la seule règle de performance qui compte : calcule hors du verrou, prends-le pour écrire, relâche-le. Un `drop(s)` explicite le relâche avant la fin du bloc. **Piège** : `let _ = m.lock();` le relâche aussitôt (chapitre 10), et `while let Ok(x) = rx.lock().unwrap().recv()` le garde pendant tout le corps de la boucle, parce qu'un temporaire dans la condition d'un `while let` vit jusqu'à la fin de l'itération. Le labo 11 te met en garde contre ce second cas, qui transforme un worker pool en file indienne.

**L'empoisonnement.** Si un thread panique en tenant le verrou, la donnée est peut-être dans un état incohérent (une mise à jour à moitié faite). Le `Mutex` est alors **empoisonné** : les `lock()` suivants rendent `Err`. C'est pour ça que `lock()` rend un `Result`, et pourquoi on voit `.lock().unwrap()` partout : un verrou empoisonné signale un bug ailleurs, et propager la panique est la réponse raisonnable.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let compte = Arc::new(Mutex::new(0));
    let c = Arc::clone(&compte);
    let r = thread::spawn(move || {
        let mut n = c.lock().unwrap();
        *n += 1;
        panic!("le thread meurt en tenant le verrou");
    })
    .join();
    println!("join : {}", if r.is_err() { "le thread a paniqué" } else { "ok" });

    match compte.lock() {
        Ok(n) => println!("valeur : {n}"),
        Err(e) => {
            println!("verrou empoisonné : {e}");
            // La donnée est toujours là : into_inner la rend quand même.
            println!("valeur récupérée : {}", *e.into_inner());
        }
    }
}
```

```
$ cargo run -q

thread '<unnamed>' (55396596) panicked at src/bin/c11_poison.rs:10:9:
le thread meurt en tenant le verrou
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
join : le thread a paniqué
verrou empoisonné : poisoned lock: another task failed inside
valeur récupérée : 1
```

Si tu sais que la donnée reste cohérente même après une panique (un compteur, un cache qu'on peut reconstruire), `lock().unwrap_or_else(|e| e.into_inner())` ignore le poison ; les tests du labo 10 le font pour un verrou qui ne protège rien.

**`RwLock<T>`** a la même forme avec deux sortes de guards : `read()` pour plusieurs lecteurs simultanés, `write()` pour un écrivain exclusif. Utile quand on lit mille fois plus qu'on n'écrit (une configuration rechargée à chaud) ; sinon, le `Mutex` est plus simple et souvent plus rapide.

**Venant de Go :** en Go, le `sync.Mutex` vit **à côté** de la donnée, dans la même struct, et rien n'empêche d'écrire `s.parCode[code]++` sans `s.mu.Lock()` : c'est le détecteur de courses qui le dira, s'il passe par là. En Rust, la donnée est **dans** le mutex, et l'oubli ne compile pas. Le `defer Unlock()` disparaît aussi : c'est le `Drop` du guard.

**Venant du C :** `pthread_mutex_t` plus la discipline de ne jamais toucher la donnée sans lui, sauf que la discipline est vérifiée. Et le verrou relâché en cas de sortie anticipée (`return`, `?`, panique) sans `goto cleanup`.

### 11.5 Les atomiques

Pour un compteur ou un drapeau, un mutex est lourd. Les types de `std::sync::atomic` (`AtomicUsize`, `AtomicU64`, `AtomicBool`, `AtomicI32`...) font une opération indivisible sur un mot, sans verrou, par une instruction du processeur. Ils se modifient à travers `&self`, donc se partagent tels quels, et un `static` atomique est un compteur global sûr, sans `unsafe` (le labo 10 en utilise un).

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::thread;

// Un compteur global, sans verrou ni unsafe.
static REQUETES: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let config = RwLock::new(HashMap::from([("timeout", 2)]));

    thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| {
                for _ in 0..1000 {
                    // Plusieurs lecteurs en même temps : read() ne bloque
                    // que si un écrivain tient le verrou.
                    let t = config.read().unwrap()["timeout"];
                    REQUETES.fetch_add(t - 1, Ordering::Relaxed);
                }
            });
        }
        s.spawn(|| {
            // Un écrivain : write() attend que tous les lecteurs aient fini.
            config.write().unwrap().insert("retries", 3);
        });
    });
    println!("requêtes : {}", REQUETES.load(Ordering::Relaxed));
    println!("config : {} clés", config.read().unwrap().len());
}
```

```
$ cargo run -q
requêtes : 8000
config : 2 clés
```

`load`, `store`, `fetch_add`, `swap`, `compare_exchange` : chaque opération prend un **`Ordering`**, qui dit ce que l'opération garantit sur les **autres** accès mémoire autour d'elle. En deux phrases : `Relaxed` ne garantit que l'atomicité de l'opération elle-même, et suffit pour un compteur ou un drapeau d'arrêt qui ne protège aucune autre donnée. Si l'atomique sert à publier autre chose (« j'ai fini d'écrire le tampon, tu peux le lire »), il faut au moins `Release` à l'écriture et `Acquire` à la lecture, et si tu ne sais pas, `SeqCst`, le plus strict, est toujours correct.

### 11.6 Les canaux : `mpsc`

`std::sync::mpsc` (*multiple producer, single consumer*) est une file typée entre threads. `channel()` rend un émetteur `Sender<T>` et un récepteur `Receiver<T>`. On **clone l'émetteur** pour avoir plusieurs producteurs ; le récepteur, lui, est unique. Quand **tous** les émetteurs sont détruits, le canal est fermé : `recv()` rend `Err`, et la boucle `for msg in rx` s'arrête.

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // mpsc : multiple producer, single consumer.
    let (tx, rx) = mpsc::channel::<String>();
    for nom in ["web1", "web2", "db1"] {
        let tx = tx.clone(); // un émetteur par producteur
        thread::spawn(move || {
            for i in 1..=2 {
                tx.send(format!("{nom} : mesure {i}")).unwrap();
                thread::sleep(Duration::from_millis(10));
            }
        }); // tx est détruit ici, à la fin du thread
    }
    drop(tx); // l'émetteur d'origine aussi, sinon la boucle ne finirait jamais

    // for sur le récepteur : s'arrête quand TOUS les émetteurs sont tombés.
    let mut n = 0;
    for msg in rx {
        println!("{msg}");
        n += 1;
    }
    println!("{n} messages, canal fermé");

    // sync_channel(0) : un rendez-vous, comme un channel Go non bufferisé.
    let (tx, rx) = mpsc::sync_channel::<u32>(0);
    let t = thread::spawn(move || {
        println!("envoi de 1 (bloque jusqu'à la réception)");
        tx.send(1).unwrap();
        println!("1 reçu de l'autre côté");
    });
    thread::sleep(Duration::from_millis(50));
    println!("  reçu {}", rx.recv().unwrap());
    t.join().unwrap();
    println!("après fermeture : {:?}", rx.recv());
}
```

```
$ cargo run -q
web1 : mesure 1
db1 : mesure 1
web2 : mesure 1
web1 : mesure 2
db1 : mesure 2
web2 : mesure 2
6 messages, canal fermé
envoi de 1 (bloque jusqu'à la réception)
  reçu 1
1 reçu de l'autre côté
après fermeture : Err(RecvError)
```

`channel()` n'est **pas borné** : `send` ne bloque jamais, la file grandit tant qu'il faut. `sync_channel(n)` est borné à `n` messages, et `send` bloque quand la file est pleine : c'est la contre-pression qui empêche un producteur rapide de remplir la mémoire. `sync_channel(0)` est un rendez-vous. Et `send` sur un canal dont le récepteur est détruit rend `Err` au lieu de paniquer.

**Piège :** le `drop(tx)` avant la boucle. L'émetteur d'origine, celui de `main`, n'est jamais utilisé, mais il existe : tant qu'il vit, le canal n'est pas fermé, et `for msg in rx` attend un septième message pour toujours. C'est l'interblocage le plus fréquent avec `mpsc`.

**Venant de Go :** `mpsc` ressemble à un channel, avec trois différences. Il n'y a **pas de `close`** explicite : le canal se ferme quand le dernier émetteur tombe, ce qui supprime d'un coup les paniques « close of closed channel » et « send on closed channel ». Il n'y a qu'**un récepteur** : pour que N workers lisent la même file, on met le `Receiver` derrière un `Arc<Mutex<_>>` (11.7), ou on prend la crate `crossbeam-channel`, dont les canaux sont *multi-consumer*. Et il n'y a **pas de `select`** dans la bibliothèque standard : `recv_timeout` couvre le cas « un message ou un délai », et `crossbeam-channel` fournit une macro `select!` complète pour attendre sur plusieurs canaux.

### 11.7 Le motif worker pool

Le même que dans le cours Go : N threads lisent des tâches dans un canal et déposent leurs résultats dans un autre. La tâche transporte un **indice**, et le résultat revient avec lui, pour être remis à sa place :

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let hotes = ["web1", "web2", "db1", "db2", "cache", "lb", "mq", "dns"];
    let (tx_taches, rx_taches) = mpsc::channel::<usize>();
    let (tx_res, rx_res) = mpsc::channel::<(usize, String)>();
    // Un seul récepteur pour N workers : on le partage derrière un Mutex.
    let rx_taches = Arc::new(Mutex::new(rx_taches));

    let debut = Instant::now();
    let mut workers = Vec::new();
    for w in 1..=3 {
        let rx = Arc::clone(&rx_taches);
        let tx = tx_res.clone();
        workers.push(thread::spawn(move || {
            loop {
                // Le verrou n'est tenu que le temps de prendre une tâche.
                let tache = rx.lock().unwrap().recv();
                let Ok(i) = tache else { break }; // canal vide et fermé : fini
                thread::sleep(Duration::from_millis(100)); // le vrai travail
                tx.send((i, format!("{} ok (worker {w})", hotes[i]))).unwrap();
            }
        }));
    }
    drop(tx_res); // seuls les workers gardent un émetteur de résultats

    for i in 0..hotes.len() {
        tx_taches.send(i).unwrap();
    }
    drop(tx_taches); // plus de tâches : les workers sortiront de leur boucle

    let mut sorties = vec![String::new(); hotes.len()];
    for (i, s) in rx_res {
        sorties[i] = s; // on remet dans l'ordre grâce à l'indice
    }
    for w in workers {
        w.join().unwrap();
    }
    for s in &sorties {
        println!("{s}");
    }
    println!("8 tâches, 3 workers : {} ms", debut.elapsed().as_millis());
}
```

```
$ cargo run -q
web1 ok (worker 2)
web2 ok (worker 1)
db1 ok (worker 3)
db2 ok (worker 1)
cache ok (worker 2)
lb ok (worker 3)
mq ok (worker 3)
dns ok (worker 1)
8 tâches, 3 workers : 307 ms
```

Huit tâches de 100 ms, trois workers : trois vagues, 300 ms. Note ce qui a disparu par rapport au Go : la goroutine qui alimente les tâches (le canal n'est pas borné, on envoie tout d'avance) et celle qui attend les workers pour fermer les résultats (le canal se ferme tout seul quand le dernier worker lâche son émetteur). Il reste deux `drop` à ne pas oublier, et `hotes` est emprunté par les threads sans `Arc`, parce que c'est un tableau de `&'static str`. Le labo 11 fait de cet exemple un vrai outil, avec `thread::scope` pour emprunter la liste des cibles.

### 11.8 L'arrêt propre

Un thread ne se tue pas de l'extérieur : il n'y a pas de `pthread_cancel` en Rust, et c'est volontaire (un thread tué en tenant un verrou laisserait tout le monde bloqué). On lui **demande** de s'arrêter, et il vérifie entre deux unités de travail. Deux façons :

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

fn main() {
    // 1. Un drapeau partagé, que le travailleur consulte entre deux tâches.
    let arret = Arc::new(AtomicBool::new(false));
    let drapeau = Arc::clone(&arret);
    let t = thread::spawn(move || {
        let mut tours = 0;
        while !drapeau.load(Ordering::Relaxed) {
            tours += 1;
            thread::sleep(Duration::from_millis(40));
        }
        tours
    });
    thread::sleep(Duration::from_millis(210));
    arret.store(true, Ordering::Relaxed);
    println!("drapeau : arrêté après {} tours", t.join().unwrap());

    // 2. Un canal : recv_timeout sert à la fois de minuterie et de signal.
    let (stop, signal) = mpsc::channel::<()>();
    let t = thread::spawn(move || {
        let mut tours = 0;
        // Timeout : on fait un tour. Message reçu ou émetteur tombé : on sort.
        while let Err(mpsc::RecvTimeoutError::Timeout) =
            signal.recv_timeout(Duration::from_millis(40))
        {
            tours += 1;
        }
        tours
    });
    thread::sleep(Duration::from_millis(210));
    drop(stop); // lâcher l'émetteur suffit : recv_timeout rend Disconnected
    println!("canal : arrêté après {} tours", t.join().unwrap());
}
```

```
$ cargo run -q
drapeau : arrêté après 5 tours
canal : arrêté après 4 tours
```

Le drapeau est le plus simple, et c'est ce que le labo 11 utilise : `sonder` regarde l'`AtomicBool` avant de se connecter, et rend « annulé » sans attendre. Sa limite : un thread bloqué dans une opération longue (un `connect_timeout` de cinq secondes, un `sleep`) ne le verra qu'à la fin de celle-ci. Le canal avec `recv_timeout` réagit tout de suite, puisque l'attente **est** la réception. Brancher le Ctrl-C sur le drapeau demande une crate (`ctrlc`, ou `signal-hook`) : la bibliothèque standard ne gère pas les signaux.

**Venant de Go :** c'est le rôle de `context.Context`, en plus rudimentaire. Il n'y a pas d'arbre de contextes qui se propage, pas de délai attaché : tu passes un `&AtomicBool` (ou un `Arc<AtomicBool>`) aux fonctions qui doivent pouvoir s'arrêter, et elles le consultent. Le vrai équivalent de `context` est l'annulation de l'async (chapitre 13), où abandonner une tâche l'arrête à son prochain `.await`.

### 11.9 La course de données impossible par construction

Mille goroutines qui incrémentent un compteur partagé, en Go :

```go
package main

import (
	"fmt"
	"sync"
)

func main() {
	compteur := 0
	var wg sync.WaitGroup
	for range 1000 {
		wg.Go(func() { compteur++ })
	}
	wg.Wait()
	fmt.Println("compteur =", compteur)
}
```

```
$ go run . ; go run .
compteur = 947
compteur = 961
$ go run -race .
==================
WARNING: DATA RACE
Read at 0x00c000012148 by goroutine 8:
  main.main.func1()
      …/racego/main.go:12 +0x2c

Previous write at 0x00c000012148 by goroutine 7:
  main.main.func1()
      …/racego/main.go:12 +0x3c
```

Le programme compile, se lance, rend un résultat faux et différent à chaque fois, et seul `-race` le dénonce, à l'exécution, à condition que la course se produise pendant ce lancement-là. Le même programme en Rust :

```rust
use std::thread;

fn main() {
    let mut compteur = 0;
    thread::scope(|s| {
        for _ in 0..1000 {
            s.spawn(|| compteur += 1);
        }
    });
    println!("compteur = {compteur}");
}
```

```
$ cargo build
error[E0499]: cannot borrow `compteur` as mutable more than once at a time
 --> src/bin/c11_race.rs:7:21
  |
5 |     thread::scope(|s| {
  |                    - has type `&'1 Scope<'1, '_>`
6 |         for _ in 0..1000 {
7 |             s.spawn(|| compteur += 1);
  |             --------^^---------------
  |             |       |  |
  |             |       |  borrows occur due to use of `compteur` in closure
  |             |       `compteur` was mutably borrowed here in the previous iteration of the loop
  |             argument requires that `compteur` is borrowed for `'1`
```

Pas de détecteur : c'est la règle du chapitre 4. Chaque fermeture veut un `&mut compteur`, et deux `&mut` vivants en même temps ne compilent pas, que ce soit dans un thread ou pas. Une course de données, c'est exactement deux accès simultanés dont l'un écrit : le vérificateur d'emprunts l'interdit déjà dans un seul thread, et `Send`/`Sync` étendent l'interdiction aux threads. La correction dit ce qu'on veut vraiment, un compteur partagé :

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn main() {
    let compteur = AtomicUsize::new(0);
    thread::scope(|s| {
        for _ in 0..1000 {
            s.spawn(|| compteur.fetch_add(1, Ordering::Relaxed));
        }
    });
    println!("compteur = {}", compteur.into_inner());
}
```

```
$ cargo run -q
compteur = 1000
```

C'est ce qu'on appelle en Rust la *fearless concurrency* : pas l'absence de bugs, l'absence de cette classe-là de bugs, la plus difficile à reproduire.

### 11.10 Ce qui reste possible

Le compilateur garantit l'absence de course de données. Il ne garantit pas que ton programme termine. **L'interblocage** (*deadlock*) reste à ta charge :

```rust
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

fn main() {
    let comptes = Mutex::new(100);
    let journal = Mutex::new(Vec::<String>::new());
    thread::scope(|s| {
        s.spawn(|| {
            let _c = comptes.lock().unwrap();
            thread::sleep(Duration::from_millis(10));
            let _j = journal.lock().unwrap(); // attend B, qui attend A
            println!("A a fini");
        });
        s.spawn(|| {
            let _j = journal.lock().unwrap();
            thread::sleep(Duration::from_millis(10));
            let _c = comptes.lock().unwrap(); // attend A, qui attend B
            println!("B a fini");
        });
    });
    println!("jamais affiché");
}
```

```
$ cargo run -q
(aucune sortie, le programme ne rend jamais la main : Ctrl-C pour l'arrêter)
```

Rien ne s'affiche, jamais, et contrairement au Go il n'y a pas de « fatal error: all goroutines are asleep - deadlock! » : les threads système attendent en silence, le processus consomme 0 % de CPU. Les remèdes sont ceux de toujours. Prendre les verrous **toujours dans le même ordre**. Tenir un verrou le moins longtemps possible, et ne jamais appeler une fonction inconnue (une fermeture fournie par l'utilisateur, un `send` sur un canal borné) en le tenant. Et ne pas reprendre un `Mutex` qu'on tient déjà dans le même thread : le `Mutex` de la bibliothèque standard n'est pas réentrant, et la documentation dit que ça bloque ou panique.

Les autres bugs logiques survivent aussi. **La famine** : un `RwLock` assailli de lecteurs peut faire attendre un écrivain longtemps. **Les situations de compétition** au sens large (*race conditions*, à distinguer des *data races*) : deux threads qui lisent un solde, calculent chacun, et réécrivent, chacun avec son verrou pris séparément pour la lecture et l'écriture, perdent une mise à jour sans qu'aucun accès mémoire ne soit concurrent. Le compilateur vérifie la mémoire, pas ta logique.

**Venant de Python :** le GIL de CPython empêche deux threads d'exécuter du bytecode en même temps : `threading` sert aux entrées-sorties, et `multiprocessing` lance des processus pour le calcul, au prix de la sérialisation des données. En Rust, les threads tournent vraiment en parallèle sur tous les cœurs, sur des données partagées sans copie. Les interblocages et les situations de compétition, eux, sont exactement les mêmes qu'en Python.

### 11.11 Quand prendre rayon

Pour du calcul pur sur une collection (transformer, filtrer, compter, trier), écrire un worker pool est du travail perdu. La crate **rayon** transforme un itérateur séquentiel en itérateur parallèle, réparti sur un pool de threads (un par cœur) avec vol de tâches :

```rust
use rayon::prelude::*;

fn est_premier(n: u64) -> bool {
    n >= 2 && (2..).take_while(|d| d * d <= n).all(|d| n % d != 0)
}

fn main() {
    let debut = std::time::Instant::now();
    let seq = (1..2_000_000u64).filter(|&n| est_premier(n)).count();
    let t_seq = debut.elapsed();
    let debut = std::time::Instant::now();
    let par = (1..2_000_000u64).into_par_iter().filter(|&n| est_premier(n)).count();
    let t_par = debut.elapsed();
    println!("{seq} premiers en {} ms, {par} en {} ms avec rayon", t_seq.as_millis(), t_par.as_millis());
}
```

```
$ cargo run -q --release
148933 premiers en 174 ms, 148933 en 24 ms avec rayon
```

Deux lignes changent : le `use`, et `into_par_iter()` (ou `par_iter()` sur une tranche). Sept fois plus rapide sur dix cœurs, et les mêmes garanties : la fermeture doit être `Send + Sync`, donc pas de course possible. Prends rayon pour le calcul ; garde les threads et les canaux de ce chapitre pour les entrées-sorties bloquantes, les tâches de fond et les services qui tournent longtemps.

### 11.12 Threads ou async : un aperçu du chapitre 13

Un thread par connexion marche bien jusqu'à quelques milliers de connexions. Au-delà, deux mégaoctets de pile réservés et un changement de contexte noyau par réveil coûtent trop. L'**async** de Rust (chapitre 13) répond au même problème que les goroutines : des **tâches** légères (quelques centaines d'octets), multiplexées par un exécutif (tokio) sur un thread par cœur, qui se suspendent à chaque `.await` au lieu de bloquer un thread. La règle de choix :

- **des threads** quand le travail est du calcul, ou quand il y a peu d'opérations bloquantes simultanées (sonder cinquante serveurs, traiter les fichiers d'un dossier, une tâche de fond) : c'est plus simple, ça marche avec n'importe quelle bibliothèque, et le débogage est celui de tout programme ;
- **async** quand il y a beaucoup d'attentes simultanées (un serveur HTTP, un client qui ouvre dix mille connexions, un proxy), ou quand l'écosystème que tu utilises est async (axum, reqwest, sqlx).

`Send`, `Sync`, `Arc` et `Mutex` restent les mêmes en async : tout ce chapitre te servira là-bas.

### 11.13 Pour le labo

Le [labo 11](../labs/11-pinger-parallele/README.md) est la transposition exacte du labo 11 du cours Go : un outil `pinger` qui sonde des ports TCP avec `TcpStream::connect_timeout`, un `parse_cible` qui comprend l'IPv6, un `sonder_tous` en worker pool (un canal de tâches qui transporte des indices, un `Arc<Mutex<Receiver>>` partagé par `parallelisme` threads lancés dans un `thread::scope`, un canal de résultats, l'ordre des cibles rétabli), un drapeau d'arrêt `AtomicBool`, et le même tableau au caractère près. Les tests montent un vrai `TcpListener` sur `127.0.0.1:0` et vérifient l'ordre des résultats avec un parallélisme de 1, 2, 10 et 0.

### À retenir

- `thread::spawn(move || ...)` lance un vrai thread système et rend un `JoinHandle` ; `join()` rend la valeur de la fermeture, ou `Err` si le thread a paniqué.
- `spawn` exige `Send + 'static` : `move` pour donner la propriété ; `thread::scope` pour des threads qui empruntent et sont attendus à la sortie.
- `Send` : transférable à un autre thread ; `Sync` : partageable par `&` ; `Rc` et `RefCell` ne le sont pas, `Arc` et `Mutex` oui.
- `Arc<Mutex<T>>` : la donnée est dans le verrou, le guard le relâche en sortant de la portée ; un verrou empoisonné signale une panique ailleurs ; `RwLock` pour beaucoup de lecteurs.
- Les atomiques pour un compteur ou un drapeau ; `Relaxed` quand rien d'autre n'en dépend, `SeqCst` si tu ne sais pas.
- `mpsc` : plusieurs émetteurs, un récepteur, fermé quand le dernier émetteur tombe ; `sync_channel(n)` pour borner ; pas de `select`, `crossbeam-channel` l'a.
- Worker pool : des indices dans un canal de tâches, le récepteur derrière un `Arc<Mutex<_>>`, des `(indice, résultat)` dans un canal de résultats, deux `drop` à ne pas oublier.
- Arrêt propre par un `AtomicBool` partagé ou un canal avec `recv_timeout` ; on ne tue pas un thread.
- Les courses de données ne compilent pas ; les interblocages, la famine et les erreurs de logique restent possibles, et l'interblocage est silencieux.
- rayon (`par_iter`) pour le calcul sur des collections ; les threads pour peu d'attentes bloquantes ; l'async du chapitre 13 pour des milliers.

---

← [10. Sous le capot : pile, tas, Box, Rc, RefCell, durées de vie, unsafe](10-sous-le-capot.md) · [Sommaire](../README.md) · [12. Fichiers, JSON, ligne de commande DevOps](12-fichiers-json-cli.md) →
