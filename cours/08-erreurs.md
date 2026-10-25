# 8. Erreurs pour de vrai : Result, ?, thiserror, anyhow, panic

*Rust de zéro à la prod : chapitre 8 sur 16.* ← [7. Traits et génériques](07-traits-et-generiques.md) · [Sommaire](../README.md) · [9. Modules, crates, Cargo, tests, documentation](09-modules-crates-tests.md) →

Depuis le chapitre 5, tu croises des `Result` partout : `parse` en rend un, `read_to_string` aussi, et tu t'en es sorti avec un `match`, un `unwrap` ou un `expect` en te promettant d'y revenir. On y est. Un programme qui tourne la nuit sur un serveur ne peut pas planter sur un fichier absent, et quand il échoue pour de bon, le message dans les logs doit dire **quoi**, **où** et **pourquoi** : « configuration de db-02 : lecture de /etc/app.conf : permission refusée », pas `called Result::unwrap() on an Err value`.

Rust, comme Go, n'a pas d'exceptions : une erreur est une valeur, renvoyée par la fonction. Mais là où Go te fait écrire `if err != nil` à chaque appel, Rust a l'opérateur `?`, qui fait la même chose en un caractère, et un système de types qui rend impossible d'oublier une erreur. Ce chapitre te donne la méthode complète : quand paniquer, quand renvoyer, comment écrire un type d'erreur propre (à la main, puis avec `thiserror`), comment ajouter du contexte dans un binaire (avec `anyhow`), et comment `main` affiche tout ça et sort avec le bon code.

### 8.1 `panic!` ou `Result` : bug ou événement

Rust sépare deux sortes d'échecs, et le choix entre les deux n'est pas une question de goût.

- Un **`Result<T, E>`** pour ce qui peut arriver en fonctionnement normal : un fichier absent, une saisie invalide, un réseau coupé, un port déjà pris. L'appelant décide quoi faire, parce que lui seul sait si c'est grave.
- Un **`panic!`** pour un **bug** : un état que le programme ne devrait jamais atteindre si le code était juste. Un index hors bornes, une division par zéro, un invariant violé. Il n'y a rien à « traiter » : il faut corriger le code.

```rust
fn moyenne(valeurs: &[f64]) -> f64 {
    assert!(!valeurs.is_empty(), "moyenne d'une liste vide");
    valeurs.iter().sum::<f64>() / valeurs.len() as f64
}

fn main() {
    for texte in ["5432", "ssh", "70000"] {
        match texte.parse::<u16>() {
            Ok(port) => println!("{texte:>6} -> port {port}"),
            Err(e) => println!("{texte:>6} -> refusé : {e}"),
        }
    }
    println!("moyenne : {}", moyenne(&[12.0, 15.0]));
    println!("moyenne : {}", moyenne(&[]));
    println!("jamais affiché");
}
```

```
$ cargo run
  5432 -> port 5432
   ssh -> refusé : invalid digit found in string
 70000 -> refusé : number too large to fit in target type
moyenne : 13.5

thread 'main' (55359037) panicked at src/main.rs:2:5:
moyenne d'une liste vide
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo $?
101
```

`parse` rend un `Result` : un texte qui n'est pas un port est un cas normal, et `main` l'affiche puis continue. `moyenne` d'une liste vide, en revanche, est une faute de l'appelant : `assert!` panique avec un message, la ligne exacte et un code de sortie 101. Un panic **déroule la pile** du thread : chaque variable locale est détruite proprement (ses `Drop` tournent, le chapitre 4 te l'a montré), puis le thread s'arrête. Pour `main`, c'est le programme qui s'arrête. `RUST_BACKTRACE=1` ajoute la pile d'appels complète.

Tu écris `panic!` (ou `assert!`, `unreachable!`, `todo!`) dans un seul cas : **un état impossible**, dont l'existence prouve un bug. Jamais pour un fichier, un argument, une réponse réseau, une donnée venue de l'extérieur : ça, c'est un `Result`.

### 8.2 `unwrap` et `expect` : où ils sont à leur place

`unwrap()` prend la valeur d'un `Ok` (ou d'un `Some`), et panique sur `Err` (ou `None`). `expect("message")` fait pareil avec ton message.

```rust
fn main() {
    let defaut: u16 = "5432"
        .parse()
        .expect("le port par défaut est un littéral valide");
    println!("défaut : {defaut}");
    let saisi: u16 = "54x2".parse().unwrap();
    println!("saisi : {saisi}");
}
```

```
$ cargo run
défaut : 5432

thread 'main' (55359122) panicked at src/main.rs:6:37:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Ils sont à leur place dans quatre situations :

1. **Les tests.** Un `unwrap` qui panique fait échouer le test, avec la ligne : c'est exactement ce qu'on veut.
2. **Les exemples et les prototypes**, où la gestion d'erreur cacherait ce qu'on montre. Ce cours en a plein, et chacun est un `unwrap` que le code de production remplacerait.
3. **Un invariant que tu peux prouver** : un littéral écrit dans le code (`"5432".parse()`), un index que tu viens de vérifier, une expression régulière constante. Préfère alors `expect`, et écris dans le message **ce que tu attendais**, pas ce qui a raté : `expect("le port par défaut est un littéral valide")`. Le jour où quelqu'un change le littéral, le panic dit pourquoi le code croyait que ça marcherait.
4. **Le démarrage d'un programme**, quand continuer n'a aucun sens et que le message est clair. Et encore : la section 8.10 montre mieux.

Partout ailleurs, chaque `unwrap` est un plantage en production qui attend son heure, avec un message (`ParseIntError { kind: InvalidDigit }`) qui ne dit ni quel fichier ni quelle ligne. `cargo clippy -- -W clippy::unwrap_used` les liste tous, pratique avant une mise en production.

### 8.3 L'opérateur `?` et la conversion `From`

`?` derrière un `Result` veut dire : si c'est `Ok(v)`, donne-moi `v` et continue ; si c'est `Err(e)`, **renvoie** `Err(e)` tout de suite à l'appelant. C'est le `if err != nil { return err }` de Go, en un caractère.

```rust
use std::fs;
use std::io;

fn premiere_ligne(chemin: &str) -> Result<String, io::Error> {
    let contenu = fs::read_to_string(chemin)?;
    Ok(contenu.lines().next().unwrap_or("").to_string())
}

fn premiere_ligne_sans_sucre(chemin: &str) -> Result<String, io::Error> {
    let contenu = match fs::read_to_string(chemin) {
        Ok(c) => c,
        Err(e) => return Err(From::from(e)),
    };
    Ok(contenu.lines().next().unwrap_or("").to_string())
}

fn main() {
    println!("{:?}", premiere_ligne("/etc/hosts"));
    println!("{:?}", premiere_ligne("/nulle/part"));
    println!("{:?}", premiere_ligne_sans_sucre("/nulle/part"));
}
```

```
$ cargo run
Ok("##")
Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
```

Les deux fonctions font exactement la même chose : la seconde montre ce que le compilateur écrit pour `?`. Note le `From::from(e)` : `?` ne renvoie pas l'erreur telle quelle, il la **convertit** vers le type d'erreur de la fonction, avec le trait `From` du chapitre 7. Ici, `io::Error` vers `io::Error`, rien ne change. Mais c'est la clé de tout le reste du chapitre, et c'est aussi la source de la première erreur que tu rencontreras. Lis un nombre dans le fichier, et le `?` du `parse` ne sait plus quoi faire :

```
$ cargo build
error[E0277]: `?` couldn't convert the error to `std::io::Error`
 --> src/main.rs:6:43
  |
4 | fn lire_port(chemin: &str) -> Result<u16, io::Error> {
  |                               ---------------------- expected `std::io::Error` because of this
5 |     let texte = fs::read_to_string(chemin)?;
6 |     let port = texte.trim().parse::<u16>()?;
  |                             --------------^ the trait `From<ParseIntError>` is not implemented for `std::io::Error`
  |                             |
  |                             this can't be annotated with `?` because it has type `Result<_, ParseIntError>`
  |
  = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```

Le message dit tout : `?` a essayé de convertir une `ParseIntError` en `io::Error`, et il n'existe pas de `From<ParseIntError> for io::Error`. La fonction peut échouer de **deux** façons, et son type d'erreur n'en décrit qu'une. Trois réponses, que les sections suivantes détaillent : un enum d'erreur à toi (8.4, puis avec `thiserror` en 8.6), une boîte qui accepte toutes les erreurs (8.5), ou `anyhow` (8.7).

`?` marche aussi sur un `Option` dans une fonction qui rend un `Option` (chapitre 6), mais on ne mélange pas : un `?` sur un `Option` dans une fonction qui rend un `Result` ne compile pas. Le pont est `ok_or` ou `ok_or_else`, qui transforme le `None` en erreur.

**Venant de Go :** l'erreur est une valeur dans les deux langages, et c'est le même modèle de pensée : on renvoie, l'appelant décide. Les différences sont toutes en faveur de Rust. Le `?` remplace trois lignes. Tu ne peux pas **oublier** de tester l'erreur : le `T` est dans le `Result`, et pour l'atteindre il faut passer par `?`, `match`, ou un `unwrap` visible. Et tu ne peux pas utiliser la valeur quand il y a une erreur, alors qu'en Go `n, err := strconv.Atoi(s)` te laisse lire `n` (qui vaut 0) même si `err` n'est pas `nil`. Le compilateur signale même un `Result` ignoré : `unused Result that must be used`.

### 8.4 Un enum d'erreur maison

Une fonction qui peut échouer de plusieurs façons rend un **enum** : une variante par cause, avec les données utiles. C'est l'enum à données du chapitre 6, au service des erreurs. Pour être une vraie erreur, il lui faut trois choses : `Debug` (dérivé), `Display` (le message pour un humain), et le trait `std::error::Error`, dont la méthode `source` donne la **cause**, l'erreur d'en dessous. Et pour que `?` convertisse tout seul, un `From` par cause.

```rust
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;

#[derive(Debug)]
enum ErreurPort {
    Lecture(io::Error),
    Format(ParseIntError),
    Reserve(u16),
}

impl fmt::Display for ErreurPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErreurPort::Lecture(_) => write!(f, "lecture du fichier de port"),
            ErreurPort::Format(_) => write!(f, "le fichier ne contient pas un port"),
            ErreurPort::Reserve(port) => write!(f, "port {port} réservé à root"),
        }
    }
}

impl Error for ErreurPort {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ErreurPort::Lecture(e) => Some(e),
            ErreurPort::Format(e) => Some(e),
            ErreurPort::Reserve(_) => None,
        }
    }
}

impl From<io::Error> for ErreurPort {
    fn from(e: io::Error) -> Self {
        ErreurPort::Lecture(e)
    }
}

impl From<ParseIntError> for ErreurPort {
    fn from(e: ParseIntError) -> Self {
        ErreurPort::Format(e)
    }
}

fn lire_port(chemin: &Path) -> Result<u16, ErreurPort> {
    let texte = fs::read_to_string(chemin)?;
    let port: u16 = texte.trim().parse()?;
    if port < 1024 {
        return Err(ErreurPort::Reserve(port));
    }
    Ok(port)
}

fn chaine(e: &dyn Error) -> String {
    let mut texte = e.to_string();
    let mut cause = e.source();
    while let Some(c) = cause {
        texte.push_str(&format!(" : {c}"));
        cause = c.source();
    }
    texte
}

fn main() -> io::Result<()> {
    let dossier = std::env::temp_dir().join("cours-rust-ch08");
    fs::create_dir_all(&dossier)?;
    for (nom, contenu) in [("ok", "5432\n"), ("texte", "pgsql\n"), ("bas", "80\n")] {
        fs::write(dossier.join(nom), contenu)?;
    }

    for nom in ["ok", "texte", "bas", "absent"] {
        match lire_port(&dossier.join(nom)) {
            Ok(port) => println!("{nom:>6} : port {port}"),
            Err(e) => println!("{nom:>6} : {}", chaine(&e)),
        }
    }

    let e = lire_port(&dossier.join("absent")).unwrap_err();
    if let ErreurPort::Lecture(io) = &e
        && io.kind() == io::ErrorKind::NotFound
    {
        println!("absent : on écrit un fichier par défaut");
    }
    println!("{e:?}");
    Ok(())
}
```

```
$ cargo run
    ok : port 5432
 texte : le fichier ne contient pas un port : invalid digit found in string
   bas : port 80 réservé à root
absent : lecture du fichier de port : No such file or directory (os error 2)
absent : on écrit un fichier par défaut
Lecture(Os { code: 2, kind: NotFound, message: "No such file or directory" })
```

- `Display` décrit **cette** couche, sans répéter la cause : « le fichier ne contient pas un port », pas « le fichier ne contient pas un port : invalid digit ». La cause, c'est `source` qui la donne. Messages en minuscules, sans point final : ils seront mis bout à bout.
- `impl Error for ErreurPort` n'a qu'une méthode à écrire, `source`, et seulement si l'erreur a une cause. `Reserve` n'en a pas : `None`.
- Les deux `From` font marcher les `?` de `lire_port` : l'`io::Error` devient `Lecture`, la `ParseIntError` devient `Format`, sans une ligne de conversion dans le corps de la fonction.
- `chaine` remonte les causes avec `source()` jusqu'à `None` et les met bout à bout. C'est ce que `anyhow` fera pour toi en section 8.7.
- L'appelant **interroge** l'erreur par `match`, ou ici par un `if let` enchaîné à une condition (édition 2024) : « si c'est une erreur de lecture, et que le fichier est absent, on crée un fichier par défaut ». Aucune analyse de texte, et le compilateur vérifie le nom des variantes.

**Venant de Go :** `errors.Is(err, os.ErrNotExist)` et `errors.As(err, &pe)` deviennent un `match` sur les variantes, ou un `if let`, et `io.kind() == ErrorKind::NotFound`. La chaîne `%w` de `fmt.Errorf` devient la méthode `source` : chaque couche garde la précédente, et on la parcourt pour afficher. Une différence de taille : en Go, `errors.As` cherche **à l'exécution** un type dans une chaîne dont tu ne sais rien ; en Rust, la signature `Result<u16, ErreurPort>` dit à l'appelant **toutes** les variantes possibles, et un `match` qui en oublie une ne compile pas.

**Venant de Python :** chaque variante est une classe d'exception, `source` est `__cause__` (le `raise NouvelleErreur() from e`), et le `match` remplace la cascade de `except FileNotFoundError:` puis `except ValueError:`. Ce qui change : l'erreur ne remonte pas toute seule. Chaque fonction qui la laisse passer le dit avec un `?`, visible à l'endroit exact où l'échec peut se produire, là où un `try` en Python couvre vingt lignes sans dire laquelle peut lever quoi.

### 8.5 `Box<dyn Error>` : la boîte qui accepte tout

Écrire un enum pour chaque fonction d'un petit script est lourd. `Box<dyn Error>`, l'objet trait du chapitre 7, est le type « n'importe quelle erreur » : toute erreur de la std ou de ta crate se convertit dedans (grâce à un `From` générique de la std), et une simple `&str` ou `String` aussi.

```rust
use std::error::Error;
use std::fs;

fn lire_port(chemin: &str) -> Result<u16, Box<dyn Error>> {
    let texte = fs::read_to_string(chemin)?;
    let port: u16 = texte.trim().parse()?;
    if port < 1024 {
        return Err(format!("port {port} réservé à root").into());
    }
    Ok(port)
}

fn main() -> Result<(), Box<dyn Error>> {
    let chemin = std::env::temp_dir().join("cours-rust-port");
    fs::write(&chemin, "pgsql\n")?;
    let chemin = chemin.to_str().ok_or("chemin non UTF-8")?;
    println!("lecture du port");
    let port = lire_port(chemin)?;
    println!("port {port}");
    Ok(())
}
```

```
$ cargo run
lecture du port
Error: ParseIntError { kind: InvalidDigit }
$ echo $?
1
```

- `lire_port` a deux `?` sur deux types d'erreur différents, et compile : les deux deviennent un `Box<dyn Error>`. `format!(...).into()` fabrique une erreur à partir d'un texte, comme `errors.New` en Go.
- `main` peut rendre `Result<(), Box<dyn Error>>` : un `?` dans `main` devient possible. Si `main` rend `Err`, le programme affiche l'erreur sur stderr et sort avec le code 1.

Regarde le message : `Error: ParseIntError { kind: InvalidDigit }`. C'est le format `Debug`, pas `Display`, sans le nom du fichier, sans la cause. Et l'appelant de `lire_port` ne peut plus distinguer un fichier absent d'un port réservé sans deviner le type caché dans la boîte. `Box<dyn Error>` est parfait pour un script de cinquante lignes et pour les exemples ; au-delà, il te manque le **contexte** et les **variantes**. Les deux crates qui suivent apportent l'un et l'autre.

### 8.6 `thiserror` : l'enum d'erreur sans le code répétitif

L'enum de la section 8.4 est juste, mais la moitié de ses lignes est mécanique : un `Display` qui recopie des messages, un `source` qui suit les variantes, deux `From`. La crate `thiserror` écrit tout ça à partir d'attributs. Elle s'ajoute avec `cargo add thiserror`, qui met dans `Cargo.toml` :

```toml
[dependencies]
thiserror = "2.0.21"
```

Voici la même erreur, avec en plus le chemin du fichier dans la variante `Lecture` :

```rust
use std::error::Error;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
enum ErreurPort {
    #[error("lecture de {}", chemin.display())]
    Lecture {
        chemin: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("le fichier ne contient pas un port")]
    Format(#[from] ParseIntError),
    #[error("port {0} réservé à root")]
    Reserve(u16),
}

fn lire_port(chemin: &Path) -> Result<u16, ErreurPort> {
    let texte = fs::read_to_string(chemin).map_err(|source| ErreurPort::Lecture {
        chemin: chemin.to_path_buf(),
        source,
    })?;
    let port: u16 = texte.trim().parse()?;
    if port < 1024 {
        return Err(ErreurPort::Reserve(port));
    }
    Ok(port)
}

fn main() {
    for chemin in ["/nulle/part/port", "/etc/hosts"] {
        let e = lire_port(Path::new(chemin)).unwrap_err();
        let cause = e.source().map(|c| c.to_string());
        println!("{e}");
        println!("  cause : {cause:?}");
        println!("  {e:?}");
    }
}
```

```
$ cargo run
lecture de /nulle/part/port
  cause : Some("No such file or directory (os error 2)")
  Lecture { chemin: "/nulle/part/port", source: Os { code: 2, kind: NotFound, message: "No such file or directory" } }
le fichier ne contient pas un port
  cause : Some("invalid digit found in string")
  Format(ParseIntError { kind: InvalidDigit })
```

- `#[derive(Error)]` implémente `std::error::Error`, et chaque `#[error("...")]` donne le `Display` de sa variante. Les champs s'insèrent par leur nom (`{chemin}`) ou leur position (`{0}`) ; pour un `PathBuf`, qui n'est pas `Display`, on passe une expression après le texte, `chemin.display()`.
- `#[source]` désigne le champ qui est la cause : c'est lui que rend `source()`. (Un champ nommé `source` est reconnu même sans l'attribut ; écris-le quand même, c'est plus clair.)
- `#[from]` fait les deux à la fois : ce champ est la cause, **et** `thiserror` écrit le `From` qui permet le `?` direct. On ne le met que sur une variante qui ne contient rien d'autre que la cause : un `From<io::Error>` ne saurait pas quel chemin mettre dans `Lecture`.
- Pour `Lecture`, on construit donc la variante à la main avec `map_err`, qui transforme l'erreur d'un `Result` avant le `?`. C'est l'endroit où l'on ajoute le contexte, ici le chemin. `map_err` puis `?` est le motif que tu écriras le plus dans une bibliothèque.
- `#[error(transparent)]` existe aussi, pour une variante qui ne fait que transmettre une erreur d'en dessous sans rien ajouter (le `Display` et le `source` de la cause passent tels quels).

Le code généré est exactement celui que tu aurais écrit : `thiserror` n'apparaît pas dans ton API publique, ne coûte rien à l'exécution, et l'utilisateur de ta bibliothèque voit un enum ordinaire qui implémente `Error`.

### 8.7 `anyhow` : le contexte dans les binaires

Dans un programme (pas une bibliothèque), on n'a presque jamais besoin de distinguer les variantes : on veut remonter jusqu'à `main` en ajoutant à chaque étage ce qu'on était en train de faire, et afficher le tout. C'est le travail d'`anyhow` (`cargo add anyhow`, version 1) :

```rust
use std::fs;
use std::io;

use anyhow::{Context, Result, anyhow, bail, ensure};

fn lire_port(chemin: &str) -> Result<u16> {
    let texte = fs::read_to_string(chemin).with_context(|| format!("lecture de {chemin}"))?;
    let port: u16 = texte
        .trim()
        .parse()
        .context("le fichier ne contient pas un port")?;
    ensure!(port >= 1024, "port {port} réservé à root");
    Ok(port)
}

fn connecter(hote: &str, chemin: &str) -> Result<String> {
    if hote.is_empty() {
        bail!("hôte vide");
    }
    let port = lire_port(chemin).with_context(|| format!("configuration de {hote}"))?;
    Ok(format!("{hote}:{port}"))
}

fn main() -> Result<()> {
    let dossier = std::env::temp_dir();
    let bon = dossier.join("port-bon");
    let faux = dossier.join("port-faux");
    fs::write(&bon, "6432\n")?;
    fs::write(&faux, "pgsql\n")?;
    let bon = bon.to_str().context("chemin non UTF-8")?;
    let faux = faux.to_str().context("chemin non UTF-8")?;

    for (hote, chemin) in [
        ("db-01", bon),
        ("db-02", faux),
        ("db-03", "/nulle/part"),
        ("", bon),
    ] {
        match connecter(hote, chemin) {
            Ok(cible) => println!("ok : {cible}"),
            Err(e) => {
                println!("{{}}   : {e}");
                println!("{{:#}} : {e:#}");
            }
        }
    }

    let e = connecter("db-03", "/nulle/part").unwrap_err();
    if let Some(io) = e.downcast_ref::<io::Error>() {
        println!("au fond : {:?}", io.kind());
    }
    println!(
        "{} étages, le dernier : {}",
        e.chain().count(),
        e.root_cause()
    );

    let e = anyhow!("délai de {} s dépassé", 5);
    println!("{e}");
    println!("---");
    Err(e).context("sonde de db-01")
}
```

```
$ cargo run
ok : db-01:6432
{}   : configuration de db-02
{:#} : configuration de db-02: le fichier ne contient pas un port: invalid digit found in string
{}   : configuration de db-03
{:#} : configuration de db-03: lecture de /nulle/part: No such file or directory (os error 2)
{}   : hôte vide
{:#} : hôte vide
au fond : NotFound
3 étages, le dernier : No such file or directory (os error 2)
délai de 5 s dépassé
---
Error: sonde de db-01

Caused by:
    délai de 5 s dépassé
```

- `anyhow::Result<T>` est un alias de `Result<T, anyhow::Error>`. `anyhow::Error` est une `Box<dyn Error>` améliorée : n'importe quelle erreur y entre avec `?`, et elle garde la chaîne des causes.
- `.context("texte")` et `.with_context(|| format!(...))` enveloppent l'erreur dans une couche de plus. `with_context` prend une fermeture, qui ne s'exécute (et n'alloue la chaîne) qu'en cas d'erreur : prends-la dès qu'il y a un `format!`. Les deux marchent aussi sur un `Option`, qu'elles transforment en erreur : `to_str().context("chemin non UTF-8")?`.
- `bail!("...")` est un `return Err(anyhow!("..."))`. `ensure!(condition, "...")` est un `bail!` conditionnel, l'`assert!` des erreurs qu'on renvoie. `anyhow!("...")` fabrique une erreur à la volée, avec le format de `format!`.
- L'affichage est ce qui rend `anyhow` indispensable. `{}` donne la couche du dessus seulement. **`{:#}` donne toute la chaîne sur une ligne**, séparée par des deux-points, exactement comme un message Go emballé avec `%w` : c'est ce que tu mets dans un log. `{:?}` donne la version sur plusieurs lignes avec `Caused by:`, et c'est ce qu'affiche un `main` qui rend un `anyhow::Result` en erreur.
- `downcast_ref::<io::Error>()` cherche un type précis dans la chaîne (c'est l'`errors.As` de Go), `chain()` parcourt les couches, `root_cause()` donne la dernière. Tu t'en sers rarement : si tu as besoin de tester des variantes, c'est que tu voulais un enum `thiserror`.

**Venant de Go :** `.with_context(|| format!("lecture de {chemin}"))?` est le `return fmt.Errorf("lecture de %s: %w", chemin, err)` de Go, et le `{:#}` produit le même message `couche: couche: cause`. Prends la même discipline que le chapitre 8 du cours Go t'a apprise : chaque couche ajoute ce que la couche du dessous ne savait pas (le chemin, l'hôte, l'opération), et rien d'autre.

### 8.8 La règle : `thiserror` dans une bibliothèque, `anyhow` dans un binaire

Les deux crates ne sont pas concurrentes, elles servent deux publics.

- **Une bibliothèque** est appelée par du code que tu ne connais pas, qui voudra peut-être réagir différemment à « fichier absent » et à « port invalide ». Elle expose donc un **enum d'erreur** (avec `thiserror`), dont les variantes font partie de son API, documentées et testées. `anyhow::Error` dans une API publique obligerait l'appelant à faire des `downcast` à l'aveugle.
- **Un binaire** est le dernier appelant. Personne au-dessus de `main` ne fera de `match`. Ce qu'il lui faut, c'est un message complet pour l'humain qui lit le terminal ou les logs : `anyhow`, `context`, `{:#}`.

Dans un même projet, les deux cohabitent : la logique dans `src/lib.rs` avec ses erreurs `thiserror`, et `src/main.rs` qui appelle la bibliothèque avec des `?` dans une fonction qui rend `anyhow::Result`, en ajoutant du contexte. Le `?` convertit l'enum de la bibliothèque en `anyhow::Error` sans rien perdre : la variante reste accessible par `downcast_ref`, et son `source` continue la chaîne. C'est exactement la structure du labo 08.

### 8.9 Les erreurs de la bibliothèque standard

Deux types d'erreur reviennent dans tous les programmes, et ils se laissent interroger sans analyser leur texte :

```rust
use std::fs;
use std::io::ErrorKind;
use std::num::IntErrorKind;

fn main() {
    for texte in ["", "-1", "99999", "8o"] {
        let e = texte.parse::<u16>().unwrap_err();
        let explication = match e.kind() {
            IntErrorKind::Empty => "vide",
            IntErrorKind::InvalidDigit => "caractère invalide",
            IntErrorKind::PosOverflow => "trop grand",
            IntErrorKind::NegOverflow => "trop petit",
            _ => "autre",
        };
        println!("{texte:>6} : {e} ({explication})");
    }

    for chemin in ["/etc/hosts", "/nulle/part", "/etc/sudoers", "/etc"] {
        match fs::read_to_string(chemin) {
            Ok(contenu) => println!("{chemin} : {} octets", contenu.len()),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => println!("{chemin} : absent, on continue"),
                ErrorKind::PermissionDenied => println!("{chemin} : refusé, relance avec sudo"),
                autre => println!("{chemin} : {autre:?} ({e})"),
            },
        }
    }
}
```

```
$ cargo run
       : cannot parse integer from empty string (vide)
    -1 : invalid digit found in string (caractère invalide)
 99999 : number too large to fit in target type (trop grand)
    8o : invalid digit found in string (caractère invalide)
/etc/hosts : 1191 octets
/nulle/part : absent, on continue
/etc/sudoers : refusé, relance avec sudo
/etc : IsADirectory (Is a directory (os error 21))
```

- `ParseIntError` (et `ParseFloatError`, et les erreurs de `parse` en général) a une méthode `kind()` qui rend un `IntErrorKind` : vide, chiffre invalide, trop grand, trop petit. Remarque `-1` en `u16` : chiffre invalide, pas « trop petit », parce qu'un type non signé ne connaît pas le signe moins. L'enum est marqué `#[non_exhaustive]` (la std se réserve d'ajouter des cas), d'où le `_` obligatoire.
- `io::Error` est l'erreur de tout ce qui touche au système : fichiers, réseau, processus. Sa méthode `kind()` rend un `ErrorKind` : `NotFound`, `PermissionDenied`, `AlreadyExists`, `ConnectionRefused`, `TimedOut`, `IsADirectory`, et une trentaine d'autres. C'est le `errno` du C, traduit en enum portable. Teste le `kind()`, jamais le texte du message, qui dépend du système.

Le motif `ErrorKind::NotFound => on crée un défaut`, `PermissionDenied => on explique`, `autre => on remonte` est celui de tout outil d'administration bien élevé. Le labo 08 le fait avec son fichier de configuration.

**Venant du C :** `ErrorKind` remplace `errno` et ses `ENOENT`, `EACCES`, `EISDIR`, avec deux améliorations : l'erreur est rendue avec le résultat (pas dans une variable globale qu'un autre appel peut écraser entre-temps), et `io::Error` garde le code d'origine (`os error 21`) pour le message. Fini aussi le `-1` qui veut dire « erreur » sauf quand c'est une valeur légitime.

### 8.10 `main` propre : stderr et codes de sortie

Un outil en ligne de commande a un contrat avec le shell qui le lance : les résultats sur **stdout**, les erreurs sur **stderr**, et un **code de sortie** non nul en cas d'échec, pour que `set -e`, `&&`, systemd ou la CI sachent que ça a raté. Le motif idiomatique tient en quinze lignes : toute la logique dans une fonction `run`, et un `main` minuscule qui traduit le résultat.

```rust
use std::process::ExitCode;

use anyhow::{Context, Result};

fn run() -> Result<()> {
    let chemin = std::env::args().nth(1).context("usage : sonde FICHIER")?;
    let texte = std::fs::read_to_string(&chemin).with_context(|| format!("lecture de {chemin}"))?;
    println!("{} lignes", texte.lines().count());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur : {e:#}");
            ExitCode::FAILURE
        }
    }
}
```

```
$ cargo run -- /etc/hosts
44 lignes
$ cargo run -- /nulle ; echo "rc=$?"
erreur : lecture de /nulle: No such file or directory (os error 2)
rc=1
$ cargo run ; echo "rc=$?"
erreur : usage : sonde FICHIER
rc=1
```

- `main` rend `ExitCode`, un type de la std. `ExitCode::SUCCESS` vaut 0, `ExitCode::FAILURE` vaut 1, et `ExitCode::from(2)` donne n'importe quel code de 0 à 255. Par convention Unix : 1 pour une erreur, 2 pour une erreur d'usage ou de configuration.
- `eprintln!` écrit sur stderr, avec `{e:#}` pour toute la chaîne de causes sur une ligne. C'est mieux que le `Error: ...` en format `Debug` qu'affiche un `main` qui rend directement un `Result`.
- `run` rend un `anyhow::Result<()>` et utilise `?` partout. Comme elle **rend** au lieu de quitter, toutes ses variables sont détruites normalement avant que `main` ne choisisse le code de sortie.

Ce dernier point est la raison de ne pas utiliser `std::process::exit` au milieu du code. `exit` termine le processus **immédiatement**, sans dérouler la pile, donc sans exécuter les `Drop` en attente :

```rust
struct Verrou;

impl Drop for Verrou {
    fn drop(&mut self) {
        println!("verrou libéré");
    }
}

fn main() {
    let _verrou = Verrou;
    if std::env::args().count() > 1 {
        eprintln!("erreur : on sort tout de suite");
        std::process::exit(3);
    }
    println!("fin normale");
}
```

```
$ cargo run
fin normale
verrou libéré
$ cargo run -- x ; echo "rc=$?"
erreur : on sort tout de suite
rc=3
```

Avec un argument, le verrou n'est jamais libéré : un fichier temporaire non effacé, un tampon d'écriture non vidé, une transaction non annulée. `process::exit` est acceptable tout en haut, quand il ne reste rien à libérer ; partout ailleurs, renvoie une erreur et laisse `main` décider.

### 8.11 `panic` n'est pas une exception

Un panic peut être rattrapé, avec `std::panic::catch_unwind` :

```rust
use std::panic;

fn main() {
    let resultat = panic::catch_unwind(|| {
        let ports: Vec<u16> = Vec::new();
        ports[0]
    });
    println!("récupéré : {}", resultat.is_err());

    let ok = panic::catch_unwind(|| 5432);
    println!("{ok:?}");
}
```

```
$ cargo run
thread 'main' (55367729) panicked at src/main.rs:6:14:
index out of bounds: the len is 0 but the index is 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
récupéré : true
Ok(5432)
```

Le panic est arrivé (le message est quand même affiché par le gestionnaire de panic), `catch_unwind` l'a arrêté et rendu sous forme de `Err`, et le programme continue. À quoi ça sert ? Aux mêmes frontières que `recover` en Go : un serveur qui ne veut pas qu'une requête qui panique tue les autres (les serveurs web de l'écosystème le font pour toi), un *thread pool* qui protège ses ouvriers, du code Rust appelé depuis du C, où un panic qui traverse la frontière serait un comportement indéfini.

Ce n'est **pas** un `try/except`. Trois raisons de ne jamais l'utiliser comme tel. Un panic signale un bug, et le rattraper pour continuer, c'est continuer avec un état faux. Un projet peut compiler avec `panic = "abort"` dans son profil `release` (binaire plus petit, chapitre 15), et alors le panic arrête tout, sans déroulement ni rattrapage possible : ton `catch_unwind` ne fait plus rien. Et les autres développeurs Rust ne s'y attendent pas : une fonction qui peut échouer rend un `Result`, c'est la convention que tout le monde lit dans la signature.

**Venant de Go :** `panic`/`recover` et `panic!`/`catch_unwind` sont jumeaux, avec la même règle d'emploi : bug d'un côté, `error` (ou `Result`) de l'autre, et le rattrapage seulement aux frontières. Le motif `defer func() { if r := recover(); r != nil { ... } }()` du cours Go s'écrit rarement en Rust : `tokio` (chapitre 13) isole déjà chaque tâche, et un panic dans une tâche lancée avec `spawn` devient une erreur pour celui qui l'attend, sans tuer les autres.

**Venant de Python :** c'est la plus grosse différence de ce chapitre. En Python, une exception remonte toute seule jusqu'à ce que quelqu'un l'attrape, et n'importe quelle ligne peut en lever une. En Rust, une erreur ne remonte que si chaque fonction sur le chemin la laisse passer avec un `?`, et la signature dit qui peut échouer. Le `traceback` est remplacé par la chaîne de contextes, que tu écris toi-même avec `context` : plus courte, et rédigée pour l'humain qui lira le log, pas pour le développeur qui lira la pile.

### 8.12 Pour le labo

Le [labo 08](../labs/08-robuste/README.md) est un lecteur de configuration `cle = valeur` qui ne plante jamais et dit toujours pourquoi. La bibliothèque définit `Config` (un port, un délai, une liste d'hôtes, un booléen) et un enum `ErreurConfig` écrit avec `thiserror` : fichier illisible (avec le chemin et la cause `io::Error`), ligne sans `=`, clé inconnue, valeur invalide (avec la cause d'origine, une `ParseIntError` ou autre, dans une `Box<dyn Error + Send + Sync>`), clé manquante. Les tests vérifient chaque variante avec `matches!` et chaque message au caractère près, sur des fichiers fournis. Le binaire, lui, utilise `anyhow` : `with_context`, la chaîne complète sur stderr avec `{:#}`, et un `ExitCode` 2 quand la configuration est en cause.

### À retenir

- `Result` pour ce qui peut arriver en production, `panic!` pour les bugs ; un panic déroule la pile et sort avec le code 101.
- `unwrap` et `expect` dans les tests, les exemples et les invariants prouvés ; `expect` dit ce qu'on attendait.
- `?` renvoie l'erreur à l'appelant après l'avoir convertie avec `From` ; `map_err` la transforme avant, `ok_or` passe d'un `Option` à un `Result`.
- Une erreur maison est un enum avec `Debug`, `Display` (cette couche seulement) et `Error::source` (la cause) ; `thiserror` écrit tout avec `#[error]`, `#[source]` et `#[from]`.
- `Box<dyn Error>` accepte toutes les erreurs, pour les scripts ; il perd les variantes et le contexte.
- `anyhow` dans les binaires : `context`, `with_context`, `bail!`, `ensure!`, `anyhow!` ; `{:#}` affiche toute la chaîne sur une ligne.
- `thiserror` dans une bibliothèque (l'appelant veut des variantes), `anyhow` dans un binaire (l'humain veut un message).
- `io::Error::kind()` et `ParseIntError::kind()` s'interrogent par `match`, jamais par le texte du message.
- `main` minuscule : une fonction `run() -> anyhow::Result<()>`, `eprintln!("erreur : {e:#}")`, et un `ExitCode` ; pas de `process::exit` au milieu du code, il saute les `Drop`.
- `catch_unwind` rattrape un panic aux frontières (serveur, FFI), jamais comme un `try/except`.

---

← [7. Traits et génériques](07-traits-et-generiques.md) · [Sommaire](../README.md) · [9. Modules, crates, Cargo, tests, documentation](09-modules-crates-tests.md) →
