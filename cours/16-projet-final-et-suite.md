# 16. Projet final et pour aller plus loin

*Rust de zéro à la prod : chapitre 16 sur 16.* ← [15. Qualité, build, release, Docker](15-qualite-build-release.md) · [Sommaire](../README.md)

Quinze chapitres, quinze labos, chacun sur un sujet. Il manque un programme où tout se tient : où le type choisi dans la configuration a des conséquences dans l'API, où des threads de sondage écrivent dans une structure que les handlers HTTP lisent au même moment, où une base SQLite vit derrière un verrou, où l'arrêt propre doit attendre tout le monde. C'est ce chapitre : d'abord le projet, son architecture et ses pièges, puis ce qu'il te reste à découvrir en Rust une fois le cours fini, et une conclusion honnête sur la place de Rust à côté de Go dans ton quotidien.

### 16.1 Le projet : sondes

Un service de supervision, comme un Uptime Kuma de poche. Tu lui donnes des cibles TCP (`hote:port`) dans un fichier TOML ; il les sonde en parallèle à intervalle régulier, enregistre chaque mesure dans SQLite, et expose l'état courant et la disponibilité sur 24 heures en JSON pour les humains, au format Prometheus pour Grafana. Il s'arrête proprement sur Ctrl-C ou `docker stop`, et tient dans une image Docker de 5,48 Mo. C'est le projet final du cours Go, avec un historique en base en plus.

Le cahier des charges complet est dans le [README du labo 16](../labs/16-projet/README.md). En résumé, ce que chaque brique réutilise :

| Brique | Ce qu'elle fait | Chapitres |
|---|---|---|
| `config.rs` | lit `sondes.toml` avec serde et toml, convertit `"500ms"` en `Duration`, valide, refuse les clés inconnues | 6, 8, 12 |
| `sonde.rs` | `sonder` et `sonder_tous` : le worker pool à threads du labo 11, synchrone, presque mot pour mot | 11 |
| `depot.rs` | la table `mesures` dans SQLite (rusqlite), le schéma par `include_str!`, `enregistrer` en transaction, `dernieres(n)`, `taux_disponibilite` en `AVG(ok)` | 14 |
| `api.rs` | axum : `/healthz`, `/api/status`, `/api/historique?cible=…&n=…`, `/metrics` au format Prometheus écrit à la main | 13 |
| `lib.rs` | l'état partagé `Etat`, un tour de sondage, la boucle à intervalle avec `select!` et `spawn_blocking` | 11, 13 |
| `main.rs` | clap avec variables d'environnement, branchement, arrêt propre des deux tâches par un canal `watch` | 12, 13, 15 |
| Livraison | `Dockerfile` multi-étapes vers `scratch`, `musl-dev` pour SQLite | 15 |
| Tests | config depuis une chaîne, Prometheus à l'octet près, dépôt en mémoire, routes par `oneshot`, sonde réelle sur un `TcpListener` local, la boucle et son arrêt | 9, 13 |

La solution fait environ 630 lignes de code hors commentaires et lignes vides, et 500 lignes de tests. Elle dépend de huit crates : tokio, axum, rusqlite, serde, serde_json, toml, clap et anyhow. Le programme Go faisait 500 lignes sans aucune dépendance. L'écart vient de la base, et du fait que Go a son serveur HTTP dans la bibliothèque standard : ce n'est pas que Rust est plus verbeux, c'est que ce problème-là est celui de Go, et qu'on l'écrit ici pour voir comment Rust s'en sort sur le terrain de l'autre.

### 16.2 Architecture et trois décisions

```
 sondes.toml ──▶ config::charger ──▶ Config ─┐
                                              ▼
                         Arc<Etat> { config, depot: Mutex<Depot>,
                                     derniers: RwLock<HashMap>, tours: AtomicU64 }
                 ┌───────────────────────────┴──────────────────────────┐
                 ▼                                                       ▼
   tokio::spawn(boucle)                                   axum::serve(routeur)
   ├ interval.tick()                                      ├ GET /healthz
   ├ spawn_blocking(tour) ──▶ sonder_tous (threads std)   ├ GET /api/status ◀── curl
   │                          └▶ etat.enregistrer ───────▶├ GET /api/historique
   └ select! sur l'arrêt                                  └ GET /metrics ◀── Prometheus

 SIGINT / SIGTERM ──▶ watch::Sender(true) ──▶ boucle sort, serveur ferme, main rend la main
```

Trois décisions à comprendre avant de coder, parce qu'elles reviennent dans tout service de ce genre.

**Le worker pool synchrone dans `spawn_blocking`, plutôt que des tâches async.** On aurait pu réécrire `sonder` avec `tokio::net::TcpStream::connect` et `tokio::time::timeout`, et lancer une tâche par cible dans un `JoinSet`. Ce serait aussi bien, et même plus léger pour dix mille cibles. Le projet fait l'autre choix exprès : le code du chapitre 11 marche, il est testé, il ne sait rien de tokio, et il se réutilise tel quel en l'appelant depuis `spawn_blocking`, qui le met sur un thread où il a le droit de bloquer. C'est la situation que tu rencontreras le plus souvent en vrai : une bibliothèque synchrone éprouvée (un client SNMP, un pilote de base, ton propre code d'il y a deux ans) dans un service async. Le pont entre les deux mondes tient en deux objets : `spawn_blocking` pour aller du async vers le bloquant, et un `AtomicBool` pour transmettre l'arrêt dans l'autre sens.

**Trois protections pour trois usages, dans le même `Etat`.** La `Config` n'est jamais modifiée après le démarrage : aucun verrou, l'`Arc` suffit. Les dernières mesures sont lues à chaque requête et écrites une fois par tour : un `RwLock`, qui laisse lire en parallèle. Le compteur de tours : un `AtomicU64`. Et le dépôt SQLite : un `Mutex`, parce qu'une `Connection` rusqlite n'est pas `Sync`. Essaie de t'en passer, et le compilateur refuse de lancer la boucle :

```
$ cargo build
error[E0277]: `RefCell<rusqlite::inner_connection::InnerConnection>` cannot be shared between threads safely
   --> src/lib.rs:127:49
    |
127 |         let tache = tokio::task::spawn_blocking(move || tour(&etat_tour, &drapeau_tour));
    |                     --------------------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `RefCell<rusqlite::inner_connection::InnerConnection>` cannot be shared between threads safely
    |
    = help: within `Etat`, the trait `Sync` is not implemented for `RefCell<rusqlite::inner_connection::InnerConnection>`
note: required because it appears within the type `Connection`
note: required because it appears within the type `Depot`
note: required because it appears within the type `Etat`
    = note: required for `Arc<Etat>` to implement `Send`
```

Lis la chaîne de bas en haut : `Arc<Etat>` doit être `Send` pour partir dans le thread, il ne l'est que si `Etat` est `Sync`, qui contient un `Depot`, qui contient une `Connection`, qui contient un `RefCell`, qui n'est pas `Sync`. En Go, ce programme aurait compilé, et deux goroutines auraient écrit en même temps dans la même connexion SQLite ; peut-être que `-race` l'aurait vu en test, peut-être pas. Ici, il ne compile pas, et le message dit où mettre le verrou.

**SQLite plutôt que la mémoire seule.** Le service Go ne gardait que le dernier résultat par cible. Celui-ci garde tout, pour pouvoir dire « disponible à 99,2 % sur 24 heures » : c'est la différence entre un tableau de bord et un outil qu'on peut montrer à quelqu'un. SQLite, parce que la base vit avec le service, dans son dossier `/data`, sans serveur à côté. La table a un index `(cible, instant)` qui sert les deux lectures, et chaque tour est écrit dans une seule transaction (un `fsync` par tour, pas un par mesure). Si un jour il faut PostgreSQL, c'est le trait `Depot` du chapitre 14 ; l'API ne changera pas.

### 16.3 Les étapes, de bas en haut

Ne commence pas par `main`. Commence par ce qui se teste sans rien lancer, et remonte. Le squelette fournit toute l'architecture (les modules, les types, le worker pool, le dépôt, les routes, `main`) et six `TODO` sur les morceaux qui font réfléchir :

1. **`config::parse_duree`.** `"500ms"`, `"10s"`, `"2m"`, `"1h"` vers une `Duration`, et un message d'erreur qui cite le texte fautif. Le reste du chargeur (serde, `#[serde(deny_unknown_fields)]`, `#[serde(rename = "cible")]` pour les tables `[[cible]]`, la validation) est fourni : lis-le, c'est le chapitre 12 et le chapitre 8 en quarante lignes.
2. **`Depot::taux_disponibilite`.** Une requête, `AVG(ok)`, et un `Option<f64>` : sur zéro ligne, `AVG` rend NULL, et « pas de mesure » n'est pas « 0 % disponible ». Le chapitre 14 en une ligne de SQL.
3. **`api::format_prometheus`.** Trois familles de métriques, avec `# HELP` et `# TYPE`, les valeurs de labels échappées. Le test compare la sortie à l'octet près, avec un nom de cible piégé (`pg "prod"\1`).
4. **Le handler `/api/status`.** L'ordre de la configuration, les `Option` de la dernière mesure (tout est `null` avant le premier tour), la disponibilité sur 24 heures. Deux verrous, pris l'un après l'autre.
5. **`tour`.** `sonder_tous`, puis `etat.enregistrer`. Trois lignes.
6. **`boucle`.** Le cœur async : `interval`, `select!` entre l'arrêt et le tick, le tour dans `spawn_blocking`, le drapeau levé à l'arrêt.

La boucle finie, dans la solution :

```rust
let drapeau = Arc::new(AtomicBool::new(false));
let mut minuterie = tokio::time::interval(etat.config.intervalle);
minuterie.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
tokio::pin!(arret);

loop {
    tokio::select! {
        () = &mut arret => break,
        _ = minuterie.tick() => {}
    }
    let etat_tour = Arc::clone(&etat);
    let drapeau_tour = Arc::clone(&drapeau);
    let tache = tokio::task::spawn_blocking(move || tour(&etat_tour, &drapeau_tour));
    tokio::select! {
        resultat = tache => match resultat {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => eprintln!("tour de sondage : {e:#}"),
            Err(e) => eprintln!("tour de sondage interrompu : {e}"),
        },
        () = &mut arret => {
            drapeau.store(true, Ordering::Relaxed);
            break;
        }
    }
}
```

Le test `boucle_sonde_puis_s_arrete` la juge : un intervalle de 50 ms, un `TcpListener` local comme cible, 180 ms d'attente, puis l'arrêt. Il exige entre deux et cinq tours (trois ou quatre en pratique), une mesure en base par tour, toutes réussies, et plus aucun tour après l'arrêt.

### 16.4 Les pièges que tu vas rencontrer

Chacun coûte une heure si on ne sait pas qu'il existe.

**Piège : la future d'arrêt consommée au premier tour.** `select!` prend ses futures par valeur. Pour attendre la même future d'arrêt à chaque tour de boucle, on passe `&mut arret`, et une référence mutable vers une future n'est une future que si elle est `Unpin`. Sans le `tokio::pin!(arret)` :

```
$ cargo build
error[E0277]: `&mut F` is not a future
   --> src/lib.rs:118:9
    |
118 | /         tokio::select! {
119 | |             () = &mut arret => break,
120 | |             _ = minuterie.tick() => {}
121 | |         }
    | |_________`&mut F` is not a future
    |
help: consider further restricting type parameter `F` with trait `Unpin`
    |
101 |     F: Future<Output = ()> + std::marker::Unpin,
    |                            ++++++++++++++++++++
```

Le conseil du compilateur marcherait (et obligerait chaque appelant à fournir une future `Unpin`), mais la bonne réponse est `tokio::pin!(arret)` : la future est épinglée sur la pile de `boucle`, elle ne bougera plus en mémoire, et `&mut arret` devient interrogeable autant de fois qu'on veut. C'est le seul endroit du cours où tu croises `Pin` ; le Async Book (16.7) explique pourquoi une future qui s'emprunte elle-même ne doit pas être déplacée.

**Piège : deux verrous imbriqués.** `/api/status` lit `derniers` (le `RwLock`) et interroge le dépôt (le `Mutex`). La boucle écrit dans les deux. Si un handler tient le premier en prenant le second pendant que la boucle fait l'inverse, chacun attend l'autre pour toujours. Rust ne détecte pas les interblocages. La règle de la solution : chaque verrou dans son bloc, relâché avant de prendre le suivant, jamais l'un dans l'autre (`enregistrer` et `status` le font, relis-les).

**Piège : le tick qui rattrape.** Par défaut, `tokio::time::interval` rattrape les ticks manqués : si un tour dure trois intervalles (des cibles muettes avec de longs timeouts), les trois ticks en retard partent d'un coup, et les tours s'enchaînent sans pause. `MissedTickBehavior::Delay` repart d'un intervalle complet après chaque tour en retard. Pour une sonde, c'est ce qu'on veut ; la validation de la config refuse d'ailleurs un timeout plus long que l'intervalle.

**Piège : l'arrêt qui attend le timeout.** Sans le drapeau, Ctrl-C pendant un tour attendrait que toutes les sondes en cours et à venir aient fini. Avec, les sondes qui n'ont pas commencé rendent « annulé » tout de suite, et celles en cours finissent dans leur timeout. Le thread bloquant n'est pas attendu par la boucle, mais il l'est par tokio quand le programme se termine : l'arrêt dure au pire le plus long timeout. C'est pour ça que les timeouts sont courts (500 ms dans l'exemple).

**Piège : les durées à 0 ms.** Une connexion TCP locale prend moins d'une milliseconde : `duree_ms` vaut 0, et la métrique ne montre rien. Ce n'est pas un bug, c'est la précision choisie. Pour des cibles locales, passe en microsecondes (`as_micros`, un `u64`, et une métrique `sondes_duree_us`) ; et souviens-toi du piège du chapitre 15 : un `u128` journalisé par tracing devient une chaîne en JSON.

**Piège : écouter sur `127.0.0.1` dans un conteneur.** Le service répond depuis le conteneur et nulle part ailleurs. Le `Dockerfile` pose `SONDES_ECOUTE=0.0.0.0:8080`, et le binaire n'a pas à le savoir : l'environnement l'emporte sur le fichier (chapitre 15).

### 16.5 Ce qu'on évalue

- `cargo test` vert, en moins d'une seconde, sans réseau extérieur ni serveur à lancer.
- `cargo clippy --all-targets -- -D warnings` muet, `cargo fmt --check` vide.
- L'arrêt propre : Ctrl-C, le journal dit `arrêt demandé` puis `arrêté après N tours`, et le processus rend la main en moins d'une seconde ; `docker stop` en 0,47 s dans la sortie du labo, pas les dix secondes d'un SIGKILL.
- `/metrics` lisible par un vrai Prometheus : `scrape_configs: [{job_name: sondes, static_configs: [{targets: ["localhost:8080"]}]}]`, et `sondes_up` apparaît dans l'explorateur.
- Une cible injoignable est une mesure `ok: false`, jamais une erreur du programme. Le service tourne quand tout est en panne autour de lui : c'est précisément à ce moment-là qu'on a besoin de lui.
- Aucun `unwrap()` sur une donnée qui vient de l'extérieur (fichier, réseau, requête HTTP). Les `expect("verrou empoisonné")` sur les verrous sont la seule exception assumée : un verrou empoisonné veut dire qu'un thread a paniqué en le tenant, et continuer serait pire qu'arrêter.

### 16.6 La version 2

Tu as un service qui tourne. Voici ce qu'il lui manque pour remplacer un vrai outil, dans l'ordre où je le ferais :

1. **Des sondes HTTP.** Un champ `type = "tcp" | "http"` dans `[[cible]]` (un `enum` serde), une sonde qui fait un `GET` et vérifie le statut, avec `reqwest::blocking::Client` pour rester dans le worker pool, ou en async dans un `JoinSet`. Puis l'expiration du certificat TLS de la cible, avec `rustls` : c'est la sonde qui évite le plus d'incidents un lundi matin.
2. **Des alertes.** Une transition `ok` vers `ko` (deux tours de suite, pour ne pas réveiller quelqu'un sur une perte de paquet) envoie un webhook : Matrix, Slack, ntfy, Telegram. reqwest, un `tokio::spawn` pour ne pas ralentir la boucle, et une table `alertes` pour ne pas renvoyer la même dix fois.
3. **La configuration rechargeable.** SIGHUP relit `sondes.toml` sans redémarrer : la `Config` passe derrière un `RwLock` (ou un canal `watch`), la boucle relit l'intervalle à chaque tour, et une config invalide est refusée en gardant l'ancienne. Pense aux mesures d'une cible qui disparaît du fichier.
4. **La journalisation structurée.** `tracing` et `tracing-subscriber` en JSON à la place des `eprintln!`, et `TraceLayer` sur le routeur (chapitres 13 et 15).
5. **Le ménage.** Une purge des mesures de plus de trente jours, une fois par heure, et un `VACUUM` de temps en temps. Sans ça, la base grossit de quelques méga-octets par jour et par cible, pour toujours.
6. **PostgreSQL et plusieurs instances.** Le trait `Depot` du chapitre 14, une implémentation `tokio-postgres` avec deadpool, et deux instances du service qui se partagent le travail. Là, les vraies questions de systèmes distribués commencent.

### 16.7 Pour aller plus loin : lire

**Le Livre.** *The Rust Programming Language* (Steve Klabnik, Carol Nichols, Chris Krycho), gratuit sur [doc.rust-lang.org/book](https://doc.rust-lang.org/book/), installé avec rustup (`rustup doc --book`), et traduit en français par la communauté : [*Le langage de programmation Rust*](https://jimskapt.github.io/rust-book-fr/). Tu peux maintenant le lire vite, en t'arrêtant sur ce que ce cours a survolé : les durées de vie (chapitre 10 du Livre), les motifs de `match` complets, les macros.

**Rustlings** ([github.com/rust-lang/rustlings](https://github.com/rust-lang/rustlings)) : une centaine de petits exercices qui ne compilent pas, à corriger dans l'ordre, avec le compilateur comme correcteur. Une heure par jour pendant une semaine, et les réflexes du vérificateur d'emprunts deviennent automatiques.

**Rust by Example** ([doc.rust-lang.org/rust-by-example](https://doc.rust-lang.org/rust-by-example/)) : chaque notion en un exemple exécutable dans le navigateur. La référence à garder ouverte quand tu as oublié une syntaxe.

**Rust for Rustaceans** (Jon Gjengset, No Starch Press, 2021) : le livre d'après. Comment concevoir une API de crate, ce que garantissent vraiment `Send` et `Sync`, `unsafe` sans avoir peur, les durées de vie variantes et contravariantes. Dense, et c'est le livre qui fait passer de « j'écris du Rust » à « je comprends pourquoi la std est faite comme ça ».

**Zero To Production in Rust** (Luca Palmieri, l'auteur de cargo-chef) : un service web complet, une lettre d'information avec ses abonnés, de la première ligne au déploiement, avec actix-web, sqlx et PostgreSQL, en passant par les tests d'intégration, la télémétrie et la CI. C'est le livre le plus proche de ce que ce cours a voulu faire, en dix fois plus long et en anglais.

**Le Async Book** ([rust-lang.github.io/async-book](https://rust-lang.github.io/async-book/)) : ce qu'il y a sous `async`/`.await`. Le trait `Future`, `poll`, le `Waker`, `Pin` et pourquoi il existe. Pas nécessaire pour écrire un service axum, indispensable le jour où tu écris ton propre combinateur ou où une erreur de `Pin` te bloque une après-midi. Le tutoriel de tokio ([tokio.rs/tokio/tutorial](https://tokio.rs/tokio/tutorial)) est son complément pratique : il fait écrire un mini Redis.

### 16.8 Les autres terrains de Rust

Ce cours est resté côté serveur et ligne de commande. Rust va ailleurs, et c'est là qu'il n'a presque pas de concurrent.

**L'embarqué.** `#![no_std]` en tête de crate : pas de bibliothèque standard, pas d'allocateur, pas de système d'exploitation, seulement `core` (les types, les traits, `Option`, `Result`, les itérateurs). Le même langage, les mêmes garanties, sur un microcontrôleur à 2 euros. [The Embedded Rust Book](https://docs.rust-embedded.org/book/) part d'une carte STM32 ; [embassy](https://embassy.dev) fait de l'async sur microcontrôleur (oui, sans OS : c'est là que l'absence d'exécuteur dans la std prend tout son sens).

**WebAssembly.** `rustup target add wasm32-unknown-unknown` pour le navigateur, avec [wasm-bindgen](https://docs.rs/wasm-bindgen) qui fait le lien avec JavaScript ; `wasm32-wasip2` pour du WebAssembly hors navigateur, exécuté par [wasmtime](https://wasmtime.dev) (lui-même écrit en Rust) ou embarqué comme système de plugins dans ton propre programme. Un binaire Wasm est un bac à sable : il ne touche ni au disque ni au réseau sans qu'on lui donne la permission.

**Python accéléré.** [PyO3](https://pyo3.rs) écrit des modules Python en Rust (une fonction annotée `#[pyfunction]` devient une fonction Python), et [maturin](https://www.maturin.rs) les construit et les publie comme des wheels : `maturin new`, `maturin develop`, et `import` depuis Python. C'est ce monde-là qui a produit uv, ruff, polars et le cœur de pydantic. Si tu as un script Python qui passe vingt minutes dans une boucle, réécrire cette boucle en Rust avec PyO3 est souvent le meilleur rapport effort sur gain de tout ce chapitre.

**Les applications de bureau.** [Tauri](https://tauri.app) : une interface en HTML et JavaScript, un cœur en Rust, et un binaire de quelques méga-octets qui utilise le moteur web du système au lieu d'embarquer Chromium comme Electron. Pour un petit outil d'administration avec une fenêtre, c'est la voie la plus courte.

### 16.9 Lire du code pour apprendre

Lire du code écrit par d'autres est la deuxième moitié de l'apprentissage. Deux projets que tu utilises déjà, écrits par des gens qui écrivent très bien.

**ripgrep** ([github.com/BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)), d'Andrew Gallant. Le dépôt est un espace de travail Cargo découpé en crates réutilisables : `crates/core` pour le binaire `rg`, `crates/ignore` pour le parcours de répertoires en parallèle qui respecte `.gitignore` (utilisé par d'autres outils, dont fd), `crates/searcher` et `crates/matcher` pour la recherche elle-même. Lis d'abord `crates/core/main.rs`, puis suis le parcours parallèle dans `crates/ignore/src/walk.rs` : c'est un worker pool, comme celui du chapitre 11, en plus sophistiqué. Les commentaires expliquent chaque choix de performance.

**fd** ([github.com/sharkdp/fd](https://github.com/sharkdp/fd)), de David Peter : le remplaçant de `find`. Un seul paquet, plus petit, plus facile à lire d'un bout à l'autre : la ligne de commande clap dans `src/cli.rs`, le parcours avec la crate `ignore` dans `src/walk.rs`, l'exécution de commandes sur les résultats (`fd -x`) dans `src/exec/`. C'est la structure à copier pour ton prochain outil en ligne de commande.

Et `cargo doc --open` dans n'importe quel projet ouvre la documentation de toutes ses dépendances, avec un lien `source` sur chaque fonction : le code de `Vec::push` ou de `tokio::spawn` est à deux clics.

### 16.10 Contribuer

Le plus court chemin pour progresser, après un projet à toi, c'est de corriger le projet de quelqu'un d'autre. Les dépôts Rust ont presque tous des issues marquées `good first issue` ou `E-easy`, et une culture de revue bienveillante : la CI te dira si ton code ne passe pas clippy avant qu'un humain ne le regarde. Commence par la documentation (un exemple qui manque, une phrase fausse dans un message d'erreur), puis un petit correctif. [This Week in Rust](https://this-week-in-rust.org) publie chaque semaine une liste « Call for Participation » de tâches choisies pour les nouveaux venus. Le forum [users.rust-lang.org](https://users.rust-lang.org) répond aux questions de débutant sans condescendance ; pose la tienne avec le message d'erreur complet, c'est ce qu'on te demandera de toute façon.

Et ce cours est vivant : une erreur, une question, une idée de labo, une issue sur le dépôt.

### 16.11 Rust ou Go, dans le quotidien d'un sysadmin

Tu as maintenant écrit le même service dans les deux langages, si tu as suivi [cours-go](https://github.com/stranix79/cours-go). Voici mon avis, sans diplomatie.

**Pour un service réseau d'infra** (un exporter, un agent, une API interne, un proxy), **Go reste mon premier choix.** Le serveur HTTP est dans la bibliothèque standard, la compilation croisée statique est gratuite, il n'y a pas de couleurs de fonctions, le programme compile en une seconde, et n'importe quel collègue le relira sans formation. Le projet `sondes` en Go faisait 500 lignes sans aucune dépendance ; en Rust, 630 lignes et huit crates, une minute de compilation à froid, et un `Pin` à comprendre. Pour ce problème-là, Go est plus court, et le plus court gagne.

**Rust gagne dès que l'une de ces conditions est vraie :**

- **La performance ou la mémoire comptent pour de vrai.** Un outil qui lit des giga-octets de journaux, un agent qui tourne sur dix mille machines (chaque méga-octet de mémoire multiplié par dix mille), un traitement qui tourne en boucle serrée. Pas de ramasse-miettes, pas de pause, une consommation mémoire prévisible. C'est ripgrep contre grep.
- **La correction compte plus que la vitesse d'écriture.** Le code qui touche aux sauvegardes, aux migrations de base, à la cryptographie, au parsing de données venues de l'extérieur. Les courses de données refusées à la compilation, le NULL et l'erreur dans les types, l'absence de nil : en Rust, la classe de bugs qu'on trouve à trois heures du matin est plus petite. Et on la paye en temps de compilation et en discussions avec le vérificateur d'emprunts, chaque jour.
- **Tu vas ailleurs que le serveur.** L'embarqué, WebAssembly, un module Python, une bibliothèque appelée depuis du C, un module du noyau Linux : Go n'y va pas, ou mal. Rust y va avec le même langage.
- **Tu lis le code de tes outils.** uv, ruff, ripgrep, fd, bat, Alacritty, le cœur de Deno, Firecracker : de plus en plus de ce qui tourne sur ta machine est écrit en Rust. Le lire, y contribuer, comprendre pourquoi il est rapide, c'est déjà une raison suffisante d'avoir suivi ce cours.

**Python et le shell gardent leur place.** Un script de vingt lignes qui enchaîne trois commandes reste un script shell. Une analyse ponctuelle de données, un notebook, une glue entre deux API qu'on lance une fois par mois, c'est Python. On n'écrit pas en Rust ce qu'on jettera la semaine prochaine.

Ma règle pratique, après les deux cours : je commence en Go quand c'est un service, en Rust quand c'est un outil qui doit être rapide ou une bibliothèque qui doit être juste, et je réécris en Rust le jour où un service Go me coûte en mémoire ou en incidents plus que la réécriture. Ce jour-là arrive moins souvent qu'on ne le lit sur Internet, et quand il arrive, tu sauras maintenant le faire.

### 16.12 Pour le labo

Le [labo 16](../labs/16-projet/README.md) est le cahier des charges détaillé de `sondes` : le format du fichier TOML, les routes et leurs réponses, les six `TODO` dans l'ordre des étapes de 16.3, dix-sept tests qui couvrent chaque module, une solution commentée ligne par ligne, et un `Dockerfile` qui produit une image de 5,48 Mo. Quand `cargo test` est vert, que clippy se tait, que `curl /metrics` répond et que Ctrl-C affiche `arrêté`, tu as écrit un service Rust de production. Compare-le avec le `hello` du labo 01.

### À retenir

- Un service se construit de bas en haut : config, sonde, dépôt, API, boucle, `main`, livraison ; chaque couche est testée avant la suivante, et `main` ne fait que brancher.
- Du code synchrone éprouvé se réutilise dans un service async par `spawn_blocking`, avec un `AtomicBool` pour lui transmettre l'arrêt.
- Chaque morceau d'état partagé a la protection de son usage : rien pour ce qui ne change pas, `RwLock` pour le lu-souvent, `Mutex` pour ce qui n'est pas `Sync`, un atomique pour un compteur ; jamais deux verrous imbriqués.
- `tokio::pin!` pour attendre la même future à chaque tour d'un `select!` ; `MissedTickBehavior::Delay` pour un intervalle qui ne rattrape pas.
- Une cible en panne est une donnée, jamais une erreur du programme ; l'arrêt propre tient dans le délai de SIGTERM.
- Après ce cours : le Livre (aussi en français), Rustlings, Rust by Example, puis Rust for Rustaceans, Zero To Production et le Async Book ; ripgrep et fd pour lire du bon code.
- Rust va là où Go ne va pas : l'embarqué avec `no_std`, WebAssembly, Python accéléré avec PyO3 et maturin, les applications avec Tauri.
- Go pour le service réseau qu'on veut court et lisible par tous, Rust pour l'outil qui doit être rapide ou le code qui doit être juste, Python et le shell pour ce qu'on jettera. Et un projet à toi qui tourne quelque part.

---

← [15. Qualité, build, release, Docker](15-qualite-build-release.md) · [Sommaire](../README.md)
