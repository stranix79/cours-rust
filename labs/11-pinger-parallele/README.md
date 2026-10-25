# Labo 11 : Pinger parallèle

**Chapitre** : [11. Concurrence : threads, channels, Arc et Mutex, Send et Sync](../../cours/11-concurrence.md)

## Objectif
Écrire `pinger`, un outil qui sonde des ports TCP en parallèle avec un nombre de sondes simultanées réglable, un délai par cible, un drapeau d'arrêt partagé, et des résultats affichés dans l'ordre des cibles quel que soit l'ordre d'arrivée. C'est le worker pool du chapitre, avec de vrais threads, deux canaux `mpsc` et un `Arc<Mutex<Receiver>>`, sur du vrai réseau. C'est aussi la transposition exacte du labo 11 du cours Go : mêmes noms, même tableau, mêmes cas de test, et tu verras ce qui change quand le compilateur vérifie le partage à ta place. Les tests montent un vrai serveur TCP local sur un port libre : ils sont déterministes, sans réseau extérieur, et tournent en moins d'une seconde.

## Consignes
1. Lis `src/lib.rs` (les types sont donnés, les fonctions sont des `todo!`) et `src/main.rs` (fourni : il lit les options à la main et affiche). Lis aussi `tests/pinger.rs` : le serveur local `serveur_local()` et l'astuce de `port_ferme()`. Lance `cargo test` : tout échoue sauf deux tests, mais ça compile.
2. `parse_cible(s: &str) -> Result<Cible, ErreurCible>` (TODO 1) : `"hote:port"` ou `"[ipv6]:port"`, port lu en `u32` puis validé entre 1 et 65535. Chaque faute a sa variante : `SansPort`, `PortInvalide(texte)`, `PortHorsPlage(n)`, `HoteVide`. Pourquoi `u32` et pas `u16` directement : pour distinguer « pas un nombre » de « trop grand ». `cargo test parse_cible`.
3. `sonder(cible, arret) -> Resultat` (TODO 2) : si `arret` est déjà levé, rends « annulé » **sans toucher au réseau** ; sinon résous l'adresse avec `ToSocketAddrs`, connecte avec `TcpStream::connect_timeout` et laisse le flux se fermer tout seul en sortant de la portée. Cas testés : un serveur local qui répond, un port fermé (`Connection refused`, immédiat), l'arrêt déjà levé sur une adresse qui ne répond jamais. `cargo test sonder_` (ça lance aussi les tests de `sonder_tous`, qui échouent encore : normal).
4. `sonder_tous(cibles, parallelisme, arret) -> Vec<Resultat>` (TODO 3) : le worker pool. Un canal de tâches qui transporte les **indices** des cibles, `parallelisme` threads (0 vaut 1) lancés dans un `thread::scope` qui lisent ces indices à tour de rôle à travers un `Arc<Mutex<Receiver<usize>>>`, sondent et envoient `(indice, Resultat)` sur un canal de résultats, et la boucle finale qui range chaque résultat à sa place. Aucune cible rend un `Vec` vide ; l'arrêt levé rend toutes les cibles en erreur, vite. Le test vérifie l'ordre avec un parallélisme de 1, 2, 10 et 0. **Piège :** si tu écris `while let Ok(i) = rx.lock().unwrap().recv()`, le verrou reste pris pendant toute la sonde et tes workers travaillent un par un ; prends la tâche dans une instruction séparée.
5. `duree_courte(d)` (TODO 4) et `format_tableau(res)` (TODO 5) : un en-tête puis une ligne par résultat, format `"{:<22} {:<4} {:>8}  {}\n"` (cible, `OK`/`KO`, durée arrondie à la milliseconde comme `"504ms"` ou `"2s"`, texte de l'erreur ou vide). Le test compare la chaîne exacte : respecte le format à l'espace près. **Piège :** `{:<22}` sur `r.cible` n'aligne rien, parce que le `Display` de `Cible` écrit avec `write!` et ignore la largeur ; passe par `to_string()`.
6. `cargo test` : tout vert. Essaie l'outil sur de vraies cibles (adapte au réseau où tu es) et regarde le code de sortie : 1 dès qu'une cible est KO, 2 sur une erreur d'usage. Lance-le avec `-p 1` puis `-p 3` sur plusieurs cibles qui ne répondent pas et compare le temps total affiché sur stderr : c'est la différence entre séquentiel et parallèle.
7. `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` : rien à redire. Il n'y a pas de `-race` à lancer : essaie de partager `cibles` entre deux threads sans `scope` ni `Arc`, et lis l'erreur ; c'est le détecteur de courses du Go, mais à la compilation.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test parse_cible           # les tests dont le nom contient parse_cible
cargo run -- -timeout 500ms 127.0.0.1:22 127.0.0.1:1 192.0.2.1:80 ; echo "code $?"
cargo run -- -p 1 -timeout 300ms 192.0.2.1:80 192.0.2.1:81 192.0.2.1:82   # séquentiel
cargo run -- -p 3 -timeout 300ms 192.0.2.1:80 192.0.2.1:81 192.0.2.1:82   # parallèle
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run -- 127.0.0.1:22)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
Avec un service local sur le port 9999 (par exemple `python3 -m http.server 9999` dans un autre terminal), un port fermé et une adresse qui ne répond jamais (192.0.2.1 est réservée à la documentation) :
```
$ cargo run -q -- -timeout 500ms 127.0.0.1:9999 127.0.0.1:1 192.0.2.1:80
CIBLE                  ETAT    DUREE  DETAIL
127.0.0.1:9999         OK         0s
127.0.0.1:1            KO         0s  Connection refused (os error 61)
192.0.2.1:80           KO      501ms  connection timed out
3 cibles en 501ms
$ echo $?
1
$ cargo run -q -- -p 1 -timeout 300ms 192.0.2.1:80 192.0.2.1:81 192.0.2.1:82 2>&1 | tail -1
3 cibles en 902ms
$ cargo run -q -- -p 3 -timeout 300ms 192.0.2.1:80 192.0.2.1:81 192.0.2.1:82 2>&1 | tail -1
3 cibles en 301ms
$ cargo run -q -- db1:70000 ; echo "code $?"
pinger : cible "db1:70000" : port 70000 hors plage (1 à 65535)
usage : pinger [-timeout 2s] [-p 8] hote:port [hote:port...]
code 2
$ (cd solution && cargo test -q --test pinger)

running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
Le message d'erreur vient du système : `os error 61` sur macOS, `os error 111` sur Linux. Les durées dépendent de ta machine, la forme non : en local, une connexion prend moins d'une demi-milliseconde et s'affiche `0s`, exactement comme le ferait le Go.

## Pour aller plus loin
- Branche le Ctrl-C sur le drapeau d'arrêt avec la crate `ctrlc` (`cargo add ctrlc`) : `ctrlc::set_handler(move || arret.store(true, Ordering::Relaxed))` avec un clone de l'`Arc`. Une sonde déjà en cours finit son délai, les suivantes rendent « annulé » tout de suite, et le tableau s'affiche quand même. C'est le `signal.NotifyContext` du Go, en trois lignes.
- Remplace le worker pool par `rayon` (`cargo add rayon`) : `cibles.par_iter().map(|c| sonder(c, &arret)).collect()` garde l'ordre tout seul, et `rayon::ThreadPoolBuilder::new().num_threads(p)` borne le parallélisme. Trente lignes deviennent trois ; décide laquelle tu préfères lire dans six mois.
- Ajoute une option `-json` qui sort les résultats en JSON avec `serde` : c'est le chapitre 12. Puis fais la même chose en async avec `tokio::net::TcpStream` et `tokio::time::timeout` : c'est le chapitre 13, et tu pourras sonder dix mille ports sans dix mille threads.
