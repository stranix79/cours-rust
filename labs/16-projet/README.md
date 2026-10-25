# Labo 16 : sondes, le projet final

**Chapitre** : [16. Projet final et pour aller plus loin](../../cours/16-projet-final-et-suite.md)

## Objectif
Écrire un service de supervision complet, `sondes` : il lit une liste de cibles TCP dans un fichier TOML, les sonde en parallèle à intervalle régulier, garde l'historique des mesures dans SQLite, et expose l'état courant en JSON pour les humains et au format Prometheus pour Grafana. Il s'arrête proprement sur Ctrl-C ou `docker stop` et tient dans une image Docker de cinq méga-octets. C'est la transposition du labo 16 du cours Go, avec une base en plus, et c'est tout le cours dans un programme : `Result` et `anyhow` (ch. 8), modules et tests (ch. 9), le worker pool à threads (ch. 11), serde et clap (ch. 12), tokio et axum (ch. 13), rusqlite (ch. 14), Docker (ch. 15).

Ce README est le cahier des charges. Les tests de `tests/sondes.rs` sont la spécification exécutable : quand ils passent tous, c'est fini.

## L'architecture (fournie)

```
labs/16-projet/
├── src/lib.rs        l'état partagé Etat, tour() et boucle()      (TODO 5, 6)
├── src/config.rs     sondes.toml : serde + toml, durées, règles   (TODO 1)
├── src/sonde.rs      Cible, Mesure, sonder, sonder_tous           (fourni : c'est le labo 11)
├── src/depot.rs      l'historique SQLite, rusqlite                (TODO 2)
├── src/api.rs        les routes axum, le format Prometheus        (TODO 3, 4)
├── src/main.rs       options, branchement, arrêt propre           (fourni)
├── migrations/001.sql  la table mesures, embarquée par include_str!
├── sondes.toml       la configuration d'exemple
├── Dockerfile, .dockerignore
└── solution/         la même chose, complète et commentée
```

Les dépendances entre modules vont dans un seul sens : `config` et `depot` connaissent `sonde`, `api` connaît `Etat`, `sonde` et `depot`, `main` connaît tout, et rien ne connaît `main`. `sonde` ne dépend de rien, pas même de tokio : c'est le worker pool synchrone du chapitre 11, appelé depuis l'async par `spawn_blocking`.

## Le fichier de configuration

```toml
intervalle = "5s"
parallelisme = 8

[[cible]]
nom = "web-local"
adresse = "127.0.0.1:8000"
timeout = "500ms"
```

| Clé | Type | Défaut | Règle |
|---|---|---|---|
| `ecoute` | chaîne | `"127.0.0.1:8080"` | adresse de l'API ; `--ecoute` ou `SONDES_ECOUTE` l'emportent |
| `base` | chaîne | `"sondes.db"` | fichier SQLite ; `--base` ou `SONDES_BASE` l'emportent |
| `intervalle` | durée (`500ms`, `10s`, `2m`, `1h`) | `10s` | positive |
| `parallelisme` | entier | `8` | au moins 1 |
| `[[cible]] nom` | chaîne | | non vide, unique |
| `[[cible]] adresse` | `hote:port` | | port entre 1 et 65535 |
| `[[cible]] timeout` | durée | `1s` | pas plus long que `intervalle` |

Une clé inconnue (`intervale`) est une erreur, pas un silence. Aucune cible est une erreur.

## Les routes

| Route | Réponse |
|---|---|
| `GET /healthz` | `200`, `{"statut":"ok","version":"1.0.0"}` |
| `GET /api/status` | `200`, `version`, `tours`, et `cibles[]` dans l'ordre du fichier : `nom`, `adresse`, `ok`, `duree_ms`, `erreur`, `instant` (secondes Unix), `disponibilite_24h` (entre 0 et 1) ; `null` avant la première mesure |
| `GET /api/historique?cible=…&n=…` | `200`, les `n` dernières mesures (20 par défaut, 1000 au plus), la plus récente d'abord ; `404` pour une cible absente du fichier, `400` pour un `n` qui n'est pas un entier |
| `GET /metrics` | `200`, `text/plain; version=0.0.4`, trois familles : `sondes_up{cible="…"}` (gauge, 0 ou 1), `sondes_duree_ms{cible="…"}` (gauge), `sondes_tours_total` (counter) |
| autre chemin | `404` ; autre méthode sur une route connue : `405` (axum le fait seul) |

## Consignes
Lance `cargo test` : ça compile, quinze tests sur dix-sept échouent. Avance de bas en haut, en relançant les tests du module que tu viens de faire.

1. **TODO 1**, `config::parse_duree(texte) -> Result<Duration, String>` : `500ms`, `10s`, `2m`, `1h` ; refuse `""`, `10`, `10 s`, `1.5s`, `10j`, `-3s`, `0s`, avec un message qui cite le texte. `cargo test duree` puis `cargo test config` : le reste du chargeur (serde, `deny_unknown_fields`, les règles) est fourni, lis-le.
2. **TODO 2**, `Depot::taux_disponibilite(cible, depuis) -> Result<Option<f64>>` : une requête `AVG(ok)`, `None` quand il n'y a aucune mesure. `cargo test depot`.
3. **TODO 3**, `api::echapper_label` et `api::format_prometheus` : le test `prometheus_a_l_espace_pres` compare la sortie complète, avec un nom de cible qui contient un guillemet et une barre oblique inverse. `cargo test prometheus` puis `cargo test labels`.
4. **TODO 4**, le handler `status` : l'ordre de la config, les `Option` de la dernière mesure, la disponibilité sur 24 heures. Deux verrous, pris l'un après l'autre et jamais l'un dans l'autre. `cargo test status`.
5. **TODO 5**, `tour(etat, arret)` : `sonder_tous` puis `etat.enregistrer`. Trois lignes.
6. **TODO 6**, `boucle(etat, arret)` : `tokio::time::interval`, `tokio::select!` sur l'arrêt et le tick, le tour dans `spawn_blocking`, et l'arrêt qui lève le drapeau des sondes. `cargo test boucle` : en 180 ms avec un intervalle de 50 ms, trois ou quatre tours, puis plus rien après l'arrêt.
7. Retire la ligne `#![allow(...)]` en haut de `lib.rs`, puis `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check`.
8. Lance-le pour de vrai (bloc ci-dessous) avec `python3 -m http.server` comme cible, regarde `/api/status` et `/metrics`, arrête le serveur Python, attends un tour, regarde `sondes_up` passer à 0. Puis Ctrl-C : le journal dit `arrêt demandé` puis `arrêté après N tours`.
9. Si tu as Docker ou Podman : construis l'image, lance-la, arrête-la avec `docker stop` (moins d'une seconde, pas les dix secondes d'un SIGKILL). Sans Docker, lis le `Dockerfile` : chaque étape est commentée.

## Comment lancer
```bash
cargo test                                   # tes tests (rouge tant que les TODO restent)
cargo test prometheus                        # un groupe de tests
(cd solution && cargo test)                  # la solution (vert)
python3 -m http.server 8000 --bind 127.0.0.1 &   # une cible qui répond
cargo run -- --config sondes.toml            # ton service
(cd solution && cargo run -- --config ../sondes.toml)
curl -s 127.0.0.1:8080/healthz
curl -s 127.0.0.1:8080/api/status | python3 -m json.tool
curl -s '127.0.0.1:8080/api/historique?cible=web-local&n=2'
curl -s 127.0.0.1:8080/metrics
docker build --build-arg SRC=solution --build-arg BIN=sondes-solution -t sondes:1.0.0 .
docker run --rm -p 8080:8080 -v sondes-data:/data sondes:1.0.0
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
Avec `python3 -m http.server 8000` qui tourne, ni SSH ni PostgreSQL en écoute sur la machine, et le service lancé avec `--ecoute 127.0.0.1:18090`. Les instants varient ; la forme, non. Les durées valent 0 ms parce que tout est local.

```
$ curl -s -i 127.0.0.1:18090/healthz
HTTP/1.1 200 OK
content-type: application/json
content-length: 33
date: Sun, 25 Oct 2026 18:12:18 GMT

{"statut":"ok","version":"1.0.0"}

$ curl -s 127.0.0.1:18090/api/status | python3 -m json.tool
{
    "version": "1.0.0",
    "tours": 2,
    "cibles": [
        {
            "nom": "web-local",
            "adresse": "127.0.0.1:8000",
            "ok": true,
            "duree_ms": 0,
            "erreur": null,
            "instant": 1792951938,
            "disponibilite_24h": 1.0
        },
        {
            "nom": "ssh-local",
            "adresse": "127.0.0.1:22",
            "ok": false,
            "duree_ms": 0,
            "erreur": "Connection refused (os error 61)",
            "instant": 1792951938,
            "disponibilite_24h": 0.0
        },
        {
            "nom": "postgres-local",
            "adresse": "127.0.0.1:5432",
            "ok": false,
            "duree_ms": 0,
            "erreur": "Connection refused (os error 61)",
            "instant": 1792951938,
            "disponibilite_24h": 0.0
        }
    ]
}

$ curl -s '127.0.0.1:18090/api/historique?cible=web-local&n=2' | python3 -m json.tool
[
    {
        "cible": "web-local",
        "instant": 1792951938,
        "ok": true,
        "duree_ms": 0,
        "erreur": null
    },
    {
        "cible": "web-local",
        "instant": 1792951933,
        "ok": true,
        "duree_ms": 0,
        "erreur": null
    }
]

$ curl -s 127.0.0.1:18090/metrics
# HELP sondes_up 1 si la cible a répondu à la dernière sonde, 0 sinon.
# TYPE sondes_up gauge
sondes_up{cible="web-local"} 1
sondes_up{cible="ssh-local"} 0
sondes_up{cible="postgres-local"} 0
# HELP sondes_duree_ms Durée de la dernière sonde, en millisecondes.
# TYPE sondes_duree_ms gauge
sondes_duree_ms{cible="web-local"} 0
sondes_duree_ms{cible="ssh-local"} 0
sondes_duree_ms{cible="postgres-local"} 0
# HELP sondes_tours_total Tours de sondage effectués depuis le démarrage.
# TYPE sondes_tours_total counter
sondes_tours_total 2

$ curl -s -i -X POST 127.0.0.1:18090/api/status | head -1
HTTP/1.1 405 Method Not Allowed
$ curl -s '127.0.0.1:18090/api/historique?cible=nope'
{"erreur":"cible \"nope\" inconnue"}
```

Le journal du service, sur stderr, puis Ctrl-C :
```
sondes 1.0.0 : 3 cibles toutes les 5s, parallélisme 8, base sondes.db, écoute sur http://127.0.0.1:18090
^Carrêt demandé
arrêté après 3 tours
```

Et dans un conteneur (ici Podman, `docker` donne la même chose) :
```
$ docker images sondes
REPOSITORY        TAG         IMAGE ID      CREATED         SIZE
localhost/sondes  1.0.0       b1e8d734e630  17 seconds ago  5.48 MB
$ docker run -d --name sondes -p 8080:8080 sondes:1.0.0
$ docker logs sondes
sondes 1.0.0 : 3 cibles toutes les 5s, parallélisme 8, base /data/sondes.db, écoute sur http://0.0.0.0:8080
$ time docker stop sondes
real    0m0,468s
$ docker logs sondes | tail -2
arrêt demandé
arrêté après 1 tours
```

## Pour aller plus loin
- Ajoute des sondes HTTP : un champ `type = "tcp" | "http"` dans `[[cible]]` (un `enum` serde avec `#[serde(rename_all = "lowercase")]`), un `url`, et une sonde qui fait un `GET` avec un `reqwest::blocking::Client` dans le worker pool, ou avec le client async du labo 13 dans un `JoinSet`. Le chapitre 16 compare les deux.
- Mets le dépôt derrière un trait `Depot` comme au labo 14 et ajoute une implémentation PostgreSQL. L'API ne change pas ; le `Mutex<Depot>` devient un `Mutex<Box<dyn Depot + Send>>`.
- Recharge la configuration sur SIGHUP sans redémarrer : la `Config` passe derrière un `RwLock` (ou un `tokio::sync::watch`), la boucle relit l'intervalle à chaque tour. Pense à ce qui arrive aux mesures d'une cible qui disparaît du fichier.
