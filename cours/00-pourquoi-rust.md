# 0. Pourquoi Rust, et quand l'utiliser (ou pas)

*Rust de zéro à la prod : chapitre 0 sur 16.* [Sommaire](../README.md) · [1. Installer, outiller, lancer](01-installer-et-outiller.md) →

En 2006, Graydon Hoare, un ingénieur de Mozilla, rentre chez lui et trouve l'ascenseur de son immeuble en panne. Encore un plantage de logiciel embarqué, écrit en C ou en C++, probablement un pointeur qui pointait là où il ne fallait pas. Il monte vingt et un étages à pied et commence un langage personnel. Mozilla le reprend en 2009 : le moteur de Firefox est un million de lignes de C++, et la moitié des failles de sécurité qu'il corrige depuis dix ans sont des erreurs de mémoire. Dépassement de tampon, lecture après libération, double libération, course entre deux threads. Des bugs que le C permet et que personne ne sait éviter à grande échelle, même chez les meilleurs.

Rust 1.0 sort en 2015 avec une promesse précise : la vitesse et le contrôle du C, sans ces bugs-là, et sans ramasse-miettes. Dix ans plus tard, le noyau Linux l'accepte pour écrire des pilotes, là où il refusait tout sauf le C depuis 1991. Android et Windows réécrivent en Rust leurs morceaux exposés. Et les outils que tu as adoptés sans y penser sont en Rust : `ripgrep` à la place de `grep`, `fd`, `bat`, `eza`, `uv` et `ruff` qui ont balayé `pip` et `flake8` en un an, Alacritty, Zed, le moteur de Deno, Turbopack, Firecracker sous les micro-VM de Lambda, Pingora qui sert une bonne partie du trafic de Cloudflare. Ce chapitre explique pourquoi ces projets l'ont choisi, ce que ça coûte, et comment tu décides, toi, pour ton prochain outil, surtout si tu as déjà Go dans la boîte.

### 0.1 Ce que Rust fait différemment

Cinq choix de conception expliquent tout le reste du cours. Ne les apprends pas par cœur, chacun a son chapitre, mais garde-les en tête : ce sont les réponses aux « mais pourquoi ? » que tu vas te poser.

**1. La possession à la place du ramasse-miettes.** Chaque valeur a un propriétaire unique. Quand le propriétaire sort de portée, la mémoire est libérée, tout de suite, par du code que le compilateur a placé pour toi. Tu peux prêter une valeur (un emprunt), en lecture à plusieurs ou en écriture à un seul, jamais les deux en même temps. Le compilateur vérifie ces règles avant de produire le binaire. Résultat : pas de `free` oublié, pas de lecture après libération, pas de ramasse-miettes qui passe, et le programme consomme exactement la mémoire dont il a besoin. C'est le chapitre 4, le plus important, et c'est ce qui fait râler tout le monde la première semaine.

**Venant du C :** c'est `malloc` et `free`, mais c'est le compilateur qui écrit les `free`, et il refuse de compiler si tu utilises un pointeur après. Les règles que tu appliquais de tête, avec Valgrind pour vérifier, deviennent des erreurs de compilation.

**Venant de Go :** le ramasse-miettes te dispensait de te demander qui possède quoi. Rust t'oblige à le dire. En échange : pas de pause, pas de `nil`, pas de course de données possible entre deux goroutines oubliées, et un agent qui tourne dans dix méga-octets au lieu de cinquante.

**Venant de Python :** tout était référence et le compteur de références faisait le ménage. Ici le ménage est décidé à la compilation, et les alias surprises (la liste que tu as modifiée sans le vouloir parce que deux variables pointaient dessus) ne compilent pas.

**2. Des abstractions à coût nul.** Un itérateur chaîné, une fermeture, un générique, un trait : tout ça disparaît à la compilation et produit le même code machine qu'une boucle écrite à la main. Tu écris du code de haut niveau, tu obtiens la vitesse du C. Il n'y a pas d'interpréteur, pas de machine virtuelle, pas d'exécutif à embarquer : un binaire Rust, c'est ton code et la bibliothèque standard, c'est tout.

**3. Un système de types qui rend les erreurs impossibles à ignorer.** Il n'y a pas de `null` : une valeur qui peut manquer est une `Option`, et le compilateur t'oblige à traiter le cas où elle manque. Il n'y a pas d'exception : une fonction qui peut échouer renvoie un `Result`, et tu ne peux pas lire la valeur sans dire ce que tu fais de l'erreur. Les énumérations portent des données, le `match` doit couvrir tous les cas, les traits remplacent l'héritage. Le compilateur attrape à midi ce que tu aurais trouvé à trois heures du matin.

**4. La concurrence sans peur.** Les mêmes règles de possession s'appliquent entre threads : une donnée partagée doit être protégée (un `Mutex`, un `Arc`), et si tu oublies, ça ne compile pas. En Go, `go test -race` trouve les courses de données à l'exécution, quand le test a la chance de les provoquer. En Rust, elles sont refusées à la compilation, toutes, sans test. Chapitre 11.

**5. Cargo.** Un seul outil pour créer un projet, compiler, tester, documenter, gérer les dépendances et publier. `cargo new`, `cargo test`, `cargo doc`, `cargo build --release`. Les dépendances viennent de crates.io, versionnées, verrouillées dans un `Cargo.lock`. Le formatage est unique (`rustfmt`), le linter est intégré (`clippy`). C'est le `go` de Go, en plus complet, et c'est l'une des raisons pour lesquelles l'écosystème a grandi si vite.

### 0.2 Le tableau qui compte

Voici comment Rust se place par rapport à ce que tu connais. Les ordres de grandeur sont ceux d'un outil en ligne de commande ou d'un service réseau ordinaire ; ils varient, mais pas d'un facteur dix.

| | C | Python | Go | Rust |
|---|---|---|---|---|
| Vitesse d'exécution | 1× (référence) | 20 à 100× plus lent | 1,2 à 2× plus lent que C | 1× à 1,2× : au niveau du C |
| Mémoire | tu gères tout | grosse, gérée | modérée, gérée (GC) | minimale, décidée à la compilation |
| Temps de compilation | secondes | aucun | secondes | dizaines de secondes à minutes |
| Déploiement | binaire + libs partagées | interpréteur + venv + paquets | un binaire statique | un binaire, statique si tu le demandes (musl) |
| Concurrence | threads POSIX, à la main | asyncio (un cœur) | goroutines + channels | threads + async, courses de données refusées à la compilation |
| Sécurité mémoire | non | oui | oui | oui, sans ramasse-miettes |
| Null | `NULL` | `None` | `nil` | n'existe pas (`Option`) |
| Gestion des erreurs | codes de retour | exceptions | valeurs `error` | `Result`, impossible à ignorer |
| Temps pour être productif | long | court | une semaine | trois à quatre semaines, le vérificateur d'emprunts en travers |
| Bibliothèque standard | minimale | énorme | énorme (HTTP, TLS, JSON) | petite et solide ; HTTP, JSON, async viennent de crates |
| Écosystème typique | systèmes, embarqué | scripts, data, IA, web | infra, réseau, CLI, cloud | outils performants, systèmes, embarqué, WASM, moteurs |

Trois lignes méritent une explication.

**Vitesse et mémoire.** Rust compile en code machine sans exécutif derrière : pas de ramasse-miettes, pas de vérification de bornes inutile (le compilateur les enlève quand il peut prouver qu'elles sont inutiles). Sur un parseur de logs, un outil de recherche, un encodeur, un proxy, Rust est au niveau du C et souvent un cran devant Go, avec deux à cinq fois moins de mémoire. Sur un service qui attend sa base de données et son réseau, la différence avec Go ne se voit pas : le temps passe ailleurs.

**Temps de compilation.** C'est le prix. Un projet Rust de taille moyenne avec `tokio`, `axum` et `serde` compile en une à trois minutes la première fois, en quelques secondes ensuite (compilation incrémentale). Un projet Go équivalent compile en cinq secondes la première fois. Ce cours partage un dossier `target/` entre tous les labos pour ne payer les dépendances qu'une fois.

**Temps pour être productif.** En Go, tu es à l'aise en une semaine. En Rust, le vérificateur d'emprunts te refuse du code que tu sais correct pendant deux à quatre semaines, jusqu'au jour où tu écris naturellement du code qu'il accepte. Ce cours est construit pour raccourcir cette période : le chapitre 4 y est entièrement consacré, avec les vrais messages d'erreur et la façon de s'en sortir.

### 0.3 Quand utiliser Rust

Voici les cas où Rust est le bon choix, presque sans discussion.

**Les outils en ligne de commande où la vitesse se sent.** Un outil qui lit des giga-octets de logs, cherche dans un dépôt, hache des fichiers, convertit des données. C'est pour ça que `ripgrep`, `fd`, `uv`, `ruff`, `delta`, `hyperfine` sont en Rust : ils tournent dix à cent fois plus vite que leurs prédécesseurs, et ça change l'usage. Si ton outil tourne en moins d'une seconde quoi qu'il arrive, Go fait aussi bien pour moins cher.

**Les bibliothèques appelées depuis Python.** `uv`, `ruff`, `pydantic-core`, `polars`, `tokenizers`, `cryptography` : le cœur en Rust, l'interface en Python, via PyO3 et maturin. Tu gardes l'écosystème Python et tu remplaces le morceau lent par un morceau cent fois plus rapide. C'est, pour un profil comme le tien, l'usage de Rust qui rapporte le plus vite.

**Les services où la mémoire et la latence comptent vraiment.** Un proxy qui tient cent mille connexions, un relais de messages, un moteur de recherche, une base de données, un agent qui doit tenir dans un conteneur de trente méga-octets. Pas de pause de ramasse-miettes, une consommation mémoire prévisible, des latences de queue plates. Discord a réécrit en Rust le service Go qui montrait des pics de latence toutes les deux minutes à cause du GC ; Cloudflare a remplacé nginx par Pingora.

**Tout ce qui parle au réseau en étant exposé.** Un parseur de protocole, un serveur TLS, un décodeur d'images, un agent qui reçoit des données d'inconnus. Les failles mémoire du C représentent environ soixante-dix pour cent des failles critiques chez Microsoft comme chez Google : Rust les supprime à la compilation. Si tu écris du code qui lit des octets venus de l'extérieur, c'est l'argument de sécurité qui tranche.

**Sans système d'exploitation, ou presque.** Microcontrôleurs (`no_std`), pilotes, noyau Linux, firmware, bootloaders. Là où C était seul, Rust est devenu le second choix accepté, avec les mêmes performances et sans les mêmes bugs. Go n'y va pas (il lui faut son exécutif).

**WebAssembly.** Un module WASM pour le navigateur, un plugin pour un proxy, une fonction en périphérie : Rust est le langage de référence de ce monde-là (`wasm-bindgen`, `wasmtime`). Petit binaire, pas d'exécutif, vitesse native.

**Le code que tu veux lire.** Même si tu n'écris jamais de Rust en production, les outils que tu utilises tous les jours sont écrits en Rust, et le code qu'une IA te propose l'est de plus en plus. Savoir lire un `match`, un `Result`, un emprunt, c'est savoir vérifier ce qu'on te donne. C'est la raison numéro un de ce cours.

### 0.4 Quand ne pas utiliser Rust

Un bon outil se reconnaît aussi à ce qu'on sait ne pas lui demander. Et ici la liste est plus longue qu'avec Go, parce que Rust coûte plus cher à écrire.

**Le script.** Renommer des fichiers, appeler une API pour voir, transformer un CSV une fois. Python ou le shell. Rust te demande un `Cargo.toml`, des types, et un `Result` à chaque ligne qui touche un fichier. Si tu ne vas pas le relancer dans un mois, ce n'est pas du Rust.

**La data, la science, l'IA.** Python, sans discussion. Rust y vit en dessous (polars, tokenizers, les noyaux de calcul), pas au-dessus.

**Le service métier ordinaire.** Une API qui lit et écrit dans PostgreSQL, un outil d'administration, un exporter Prometheus, un agent qui lance des commandes. Go le fait en moitié moins de lignes, compile en cinq secondes, et n'importe quel sysadmin peut y corriger une ligne sans apprendre la possession. Pour pgCellar, un outil de sauvegarde PostgreSQL avec une interface web, le choix a été Go précisément pour ça : les gens qui vont le lire et le patcher sont des administrateurs, pas des compilateurs. Rust aurait gagné dix pour cent de mémoire et perdu la moitié des contributeurs possibles.

**L'équipe qui ne le connaît pas, et le projet qui dure trois semaines.** Le coût d'apprentissage est réel (trois à quatre semaines pour être à l'aise, ce cours ou pas). Pour un projet court dans une équipe Go ou Python, reste dans la langue de l'équipe.

**Le prototype.** Quand tu ne sais pas encore quelle forme aura le programme, Rust te fait payer chaque changement de structure : les types et les emprunts se propagent. Prototype en Python ou en Go, réécris en Rust le morceau qui doit être rapide quand tu sais lequel c'est.

**Les interfaces graphiques.** Tauri et egui existent et marchent, mais une app macOS reste du Swift et une app web reste du TypeScript. Rust peut servir l'API derrière, ou tourner dans le navigateur en WASM pour le morceau lourd.

**Et quand on réécrit tout.** La tentation classique après ce cours : tout refaire en Rust. Non. Réécris ce qui est lent, ce qui est exposé, ce qui fuit. Garde le reste.

Les coûts, dits franchement : la courbe d'apprentissage (le vérificateur d'emprunts), les temps de compilation, l'async qui est plus difficile qu'en Go (plusieurs exécuteurs, des types de futures qui s'allongent, des messages d'erreur de deux écrans), une bibliothèque standard volontairement petite (il faut choisir ses crates), et un écosystème qui bouge encore (une crate majeure change d'API tous les deux ans). Rien de rédhibitoire, mais rien de gratuit.

### 0.5 Rust ou Go : la règle de décision

Tu as les deux. Réponds à ces questions dans l'ordre ; la première qui dit oui décide.

1. C'est jetable, ou ça tient en vingt lignes ? **Shell ou Python.**
2. C'est de la data, de la science, de l'IA ? **Python** (avec du Rust dessous si c'est lent, mais ce n'est pas toi qui l'écris).
3. Ça tourne sans système d'exploitation, sur un microcontrôleur, dans un noyau, dans le navigateur en WASM ? **Rust** (ou C).
4. Ça lit des octets venus d'inconnus, ça doit tenir cent mille connexions, ou chaque milliseconde et chaque méga-octet se voient sur la facture ? **Rust.**
5. C'est un morceau lent d'un programme Python qu'il faut accélérer ? **Rust**, via PyO3.
6. C'est un outil CLI, un service réseau, un agent, un exporter, quelque chose que des sysadmins vont lire et patcher ? **Go.**
7. Aucune des réponses ci-dessus ? Prends le langage que ton équipe lit le mieux. Entre les deux, Go est le choix par défaut ; Rust est le choix quand tu as une raison.

Le cas fréquent où les deux marchent : un outil en Go qui marche bien mais qui, sur ton plus gros jeu de données, met dix minutes et deux giga-octets. C'est exactement le moment de réécrire ce morceau-là en Rust, pas tout l'outil.

### 0.6 Ce que Rust n'est pas

Quatre idées reçues à évacuer avant de commencer.

**« Rust, c'est pour les génies. »** Non. C'est pour les gens qui acceptent de se faire disputer par le compilateur pendant trois semaines. Les règles tiennent en une page (chapitre 4). Ce qui est difficile, c'est de désapprendre les habitudes du C (les pointeurs partout) et de Python (les références partout) ; une fois que c'est fait, le compilateur devient un collègue qui relit chaque ligne.

**« Rust est toujours plus rapide que Go. »** Sur du calcul, de l'analyse de texte, de la mémoire : oui, nettement. Sur un service qui attend sa base de données : non, les deux attendent pareil. Mesure avant de choisir (chapitre 15 montre comment).

**« `unsafe`, c'est du C déguisé, donc Rust ne garantit rien. »** `unsafe` débloque quatre choses précises (déréférencer un pointeur brut, appeler une fonction `unsafe` ou une fonction C, lire une variable globale mutable, implémenter un trait `unsafe`) dans un bloc délimité que tu peux relire et auditer. Tout le reste du programme garde ses garanties. La bibliothèque standard en contient, ton code en contiendra rarement, et le chapitre 10 montre quand.

**« Rust remplace le C partout. »** Il remplace le C dans le nouveau code exposé et performant. Les millions de lignes existantes de C restent, et Rust sait leur parler sans frais (FFI, chapitre 10). Le noyau Linux ne se réécrit pas ; il accepte de nouveaux pilotes en Rust.

### 0.7 Comment lire ce cours

Le cours suit le fil du cours Go, parce que tu le connais : **chapitre puis labo**. Lis le chapitre, fais le labo (un squelette à compléter avec des `// TODO`, des tests `cargo test` qui disent si c'est bon, une solution commentée), passe au suivant.

Les chapitres 1 à 9 posent le langage. Ils sont écrits pour quelqu'un qui vient du C, de Python et de Go : chaque notion est comparée à ce que tu connais, et les pièges de traduction sont signalés. Le chapitre 4 est le cœur : prends ton temps, fais le labo deux fois si besoin.

Les chapitres 10 et 11 font de toi quelqu'un qui *comprend* Rust : ce qui se passe en mémoire, les pointeurs intelligents, `unsafe`, puis la concurrence avec ses garanties.

Les chapitres 12 à 16 sont ton quotidien : outils en ligne de commande, fichiers et JSON, HTTP et async, bases de données, qualité, build, Docker, et un projet final qui rassemble tout.

Le code est en Rust 1.93, édition 2024. Tout ce qui est dans un bloc `rust` a été compilé et exécuté ; la sortie est donnée juste après. Quand une notion a un équivalent en C, en Python ou en Go, il est donné dans un encadré **Venant du C**, **Venant de Python** ou **Venant de Go**. Quand Rust fait quelque chose qui te paraîtra bizarre, c'est signalé par **Piège**. Quand le compilateur refuse quelque chose, son message est cité tel quel : apprendre Rust, c'est apprendre à lire ces messages, qui sont les meilleurs du métier.

Compte une à deux heures par chapitre, le double pour le chapitre 4. Ne saute pas les labos : lire du Rust donne l'impression de comprendre, écrire du Rust et se faire refuser par le compilateur fait comprendre.

### À retenir

- Rust est compilé, sans ramasse-miettes, aussi rapide que le C, et il refuse à la compilation les erreurs de mémoire et les courses de données.
- Le prix : une courbe d'apprentissage de trois à quatre semaines (la possession) et des temps de compilation plus longs.
- Utilise Rust pour : les outils où la vitesse se sent, les morceaux lents d'un programme Python, les services où la mémoire et la latence comptent, le code exposé au réseau, l'embarqué et le WASM.
- N'utilise pas Rust pour : le script, la data et l'IA, le service métier ordinaire (c'est du Go), le prototype, et pour tout réécrire.
- Entre Go et Rust : Go par défaut, Rust quand tu as une raison (vitesse, mémoire, sécurité, sans OS).
- Même sans en écrire, savoir lire du Rust te sert tous les jours : tes outils et le code qu'on te génère sont en Rust.
- Pas de `null`, pas d'exceptions, pas d'héritage, pas de ramasse-miettes : chaque absence est un choix, et chacune a son chapitre.

---

[Sommaire](../README.md) · [1. Installer, outiller, lancer](01-installer-et-outiller.md) →
