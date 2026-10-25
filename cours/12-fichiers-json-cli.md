# 12. Fichiers, JSON, ligne de commande DevOps

*Rust de zéro à la prod : chapitre 12 sur 16.* ← [11. Concurrence : threads, channels, Arc et Mutex, Send et Sync](11-concurrence.md) · [Sommaire](../README.md) · [13. HTTP et async : tokio, reqwest, axum](13-http-et-async.md) →

C'est le chapitre du quotidien. Lire un fichier de configuration, parcourir un dossier pour trouver ce qui remplit le disque, filtrer un journal, lancer `pg_dump` et vérifier son code de retour, sortir du JSON pour `jq`, accepter des options sans écrire cent lignes d'analyse d'arguments. La bibliothèque standard couvre les fichiers, les chemins, l'environnement et les processus ; deux crates, `clap` et `serde`, couvrent la ligne de commande et les formats, et elles sont si répandues qu'on peut les considérer comme faisant partie du langage. À la fin, tu sais écrire en Rust les outils que tu écrivais en shell ou en Python, avec des erreurs qui disent quel fichier manque et des binaires qui se copient sur un serveur sans rien installer.

### 12.1 `std::fs` : lire, écrire, parcourir

Les fonctions de `std::fs` font une chose chacune et rendent toutes un `io::Result` :

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Parcours récursif : la taille totale d'un dossier, liens non suivis.
fn taille_totale(dir: &Path) -> io::Result<u64> {
    let mut total = 0;
    for entree in fs::read_dir(dir)? {
        let entree = entree?;
        let genre = entree.file_type()?;
        if genre.is_dir() {
            total += taille_totale(&entree.path())?;
        } else if genre.is_file() {
            total += entree.metadata()?.len();
        }
    }
    Ok(total)
}

fn main() -> io::Result<()> {
    let racine = std::env::temp_dir().join(format!("demo-fs-{}", std::process::id()));
    let conf = racine.join("etc").join("app");
    fs::create_dir_all(&conf)?; // mkdir -p
    fs::write(conf.join("app.toml"), "port = 8080\n")?;
    fs::write(racine.join("etc").join("hosts"), "127.0.0.1 localhost\n")?;

    let texte = fs::read_to_string(conf.join("app.toml"))?;
    println!("app.toml : {texte:?}");

    let mut noms: Vec<PathBuf> = fs::read_dir(racine.join("etc"))?
        .map(|e| e.map(|e| e.path()))
        .collect::<io::Result<_>>()?;
    noms.sort();
    for n in &noms {
        let m = fs::metadata(n)?;
        let genre = if m.is_dir() { "dossier" } else { "fichier" };
        println!("{genre:<8} {:>3} o  {}", m.len(), n.file_name().unwrap().to_string_lossy());
    }
    println!("total : {} octets", taille_totale(&racine)?);

    fs::remove_file(conf.join("app.toml"))?;
    fs::remove_dir_all(&racine)?; // rm -rf : à manier avec soin
    println!("nettoyé : {}", !racine.exists());
    Ok(())
}
```

```
$ cargo run -q
app.toml : "port = 8080\n"
dossier   96 o  app
fichier   20 o  hosts
total : 32 octets
nettoyé : true
```

- `read_to_string` et `write` lisent et écrivent un fichier entier en une ligne. Pour un fichier de configuration ou un petit JSON, c'est ce qu'il faut ; pour un journal de 10 Go, c'est la section 12.3. `fs::read` rend des octets (`Vec<u8>`) au lieu d'une `String` : `read_to_string` échoue sur un fichier qui n'est pas de l'UTF-8 valide.
- `create_dir_all` est `mkdir -p` : il crée les parents et ne dit rien si le dossier existe. `create_dir` échoue s'il existe déjà, ce qui sert parfois de verrou grossier.
- `read_dir` rend un itérateur de `io::Result<DirEntry>` : **chaque entrée** peut échouer à part (un fichier supprimé entre la liste et la lecture). D'où le `entree?` en tête de boucle, et le `collect::<io::Result<_>>()?` qui transforme « un itérateur de Result » en « un Result de Vec », en s'arrêtant à la première erreur (chapitre 8). L'ordre de `read_dir` est celui du système de fichiers, c'est-à-dire n'importe lequel : trie si tu affiches.
- `metadata` suit les liens symboliques, `symlink_metadata` et `DirEntry::file_type` non. Pour un parcours récursif, prends `file_type` : un lien vers `/` ne doit pas te faire parcourir tout le disque. La taille d'un dossier (96 ici) est celle de l'entrée de répertoire elle-même, pas de son contenu ; c'est pourquoi `taille_totale` ne compte que les fichiers.
- `remove_file`, `remove_dir` (vide seulement), `remove_dir_all` (récursif), `rename` (atomique sur un même système de fichiers : écris dans un fichier temporaire puis renomme, et personne ne lira jamais un fichier à moitié écrit), `copy`.

Pour un parcours récursif sérieux (des millions de fichiers, des boucles de liens, un filtre `.gitignore`), la crate `walkdir`, ou `ignore` (celle de ripgrep) ; la fonction de dix lignes ci-dessus suffit pour un outil maison, et c'est elle que le labo te fait écrire.

**Venant de Go :** `os.ReadFile`, `os.WriteFile`, `os.MkdirAll`, `os.ReadDir`, `os.Stat`, `os.RemoveAll` ont leurs jumeaux exacts. La différence est le type d'erreur : un `io::Error` avec un `kind()` à comparer (12.3), au lieu d'un `error` à tester avec `errors.Is(err, fs.ErrNotExist)`. Et le message d'un `io::Error` ne contient **pas** le chemin : « No such file or directory (os error 2) », point. C'est ton travail de l'ajouter, avec le `with_context` d'anyhow (chapitre 8, et labo 12).

**Venant du C :** `fopen`, `fread`, `opendir`, `readdir`, `stat`, `unlink`, sans `fclose` (le `Drop` du `File`) et sans `errno` (le `Result`).

### 12.2 `Path` et `PathBuf`

Un chemin n'est pas une chaîne. Sous Unix, un nom de fichier est une suite d'octets quelconques sauf `/` et `\0`, pas forcément de l'UTF-8 ; sous Windows, c'est de l'UTF-16 mal formé autorisé. Rust a donc deux types dédiés, dans le même rapport que `&str` et `String` : `&Path` emprunté, `PathBuf` possédé. Les fonctions prennent `impl AsRef<Path>`, ce qui leur fait accepter `&str`, `String`, `&Path` et `PathBuf` indifféremment.

```rust
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("/var/log");
    let fichier: PathBuf = base.join("nginx").join("access.log.1.gz");
    println!("chemin     : {}", fichier.display());
    println!("parent     : {:?}", fichier.parent());
    println!("file_name  : {:?}", fichier.file_name());
    println!("file_stem  : {:?}", fichier.file_stem());
    println!("extension  : {:?}", fichier.extension());
    println!("existe     : {}", fichier.exists());
    println!("absolu     : {}", fichier.is_absolute());

    let mut p = PathBuf::from("rapport");
    p.set_extension("json");
    println!("set_ext    : {}", p.display());
    // join avec un chemin absolu REMPLACE tout : piège classique.
    println!("piège      : {}", base.join("/etc/passwd").display());
    let composants: Vec<_> = fichier.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    println!("composants : {composants:?}");
}
```

```
$ cargo run -q
chemin     : /var/log/nginx/access.log.1.gz
parent     : Some("/var/log/nginx")
file_name  : Some("access.log.1.gz")
file_stem  : Some("access.log.1")
extension  : Some("gz")
existe     : false
absolu     : true
set_ext    : rapport.json
piège      : /etc/passwd
composants : ["/", "var", "log", "nginx", "access.log.1.gz"]
```

`file_name`, `extension` et `parent` rendent des `Option` : `/` n'a pas de nom, `Makefile` pas d'extension. Ils rendent des `&OsStr`, pas des `&str` : `.to_str()` rend `Option<&str>` (None si ce n'est pas de l'UTF-8), `.to_string_lossy()` remplace les octets invalides. Un `Path` ne s'affiche pas avec `{}` directement : `.display()` le formate pour un humain, en acceptant la perte.

**Piège :** `base.join("/etc/passwd")` rend `/etc/passwd`. Un `join` avec un chemin absolu remplace tout. Avec un littéral, clippy le voit (`join_absolute_paths`) ; avec un nom de fichier venu d'une requête HTTP ou d'une archive, personne ne le voit, et c'est une faille de traversée de répertoire. Vérifie que le morceau n'est pas absolu et ne contient pas `..` (`Path::components` le dit) avant de joindre quoi que ce soit qui vient de l'extérieur.

`exists()` répond `false` aussi quand tu n'as pas le droit de regarder. Pour savoir pourquoi, `fs::metadata(p)` et son erreur. Et ne teste pas `exists()` avant d'ouvrir : entre les deux, le fichier peut apparaître ou disparaître. Ouvre, et traite l'erreur.

**Venant de Python :** c'est `pathlib`, presque nom pour nom (`Path("/var/log") / "nginx"` devient `.join("nginx")`, `.suffix` devient `.extension()`, `.stem` `.file_stem()`, `.parent` `.parent()`). La différence est l'`Option` : Python rend `""` pour une extension absente, Rust rend `None`.

### 12.3 Lire et écrire en flux, et les erreurs d'entrée-sortie

Pour un gros fichier, on ne charge pas tout : on lit ligne par ligne avec un `BufReader`, et on écrit par blocs avec un `BufWriter`.

```rust
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write};

fn compter_erreurs(chemin: &str) -> io::Result<usize> {
    let f = File::open(chemin)?;
    let mut n = 0;
    // BufReader lit par blocs de 8 Kio ; lines() découpe sans charger le fichier.
    for ligne in BufReader::new(f).lines() {
        if ligne?.contains(" ERROR ") {
            n += 1;
        }
    }
    Ok(n)
}

fn main() -> io::Result<()> {
    let chemin = env::temp_dir().join(format!("demo-io-{}.log", std::process::id()));
    {
        // BufWriter regroupe les petites écritures : un appel système par
        // bloc au lieu d'un par ligne. Le Drop vide le tampon.
        let mut w = BufWriter::new(File::create(&chemin)?);
        for i in 0..10_000 {
            let niveau = if i % 1000 == 0 { "ERROR" } else { "INFO" };
            writeln!(w, "2026-10-25 01:02:03 {niveau} requête {i}")?;
        }
        w.flush()?; // explicite : une erreur d'écriture serait perdue dans le Drop
    }
    println!("erreurs : {}", compter_erreurs(chemin.to_str().unwrap())?);
    std::fs::remove_file(&chemin)?;

    for f in ["/nulle/part.log", "/etc/sudoers"] {
        match File::open(f) {
            Ok(_) => println!("{f} : ouvert"),
            Err(e) if e.kind() == ErrorKind::NotFound => println!("{f} : absent ({e})"),
            Err(e) if e.kind() == ErrorKind::PermissionDenied => println!("{f} : interdit ({e})"),
            Err(e) => return Err(e),
        }
    }

    match env::var("DSN_DEMO") {
        Ok(v) => println!("DSN_DEMO = {v}"),
        Err(e) => println!("DSN_DEMO : {e}"),
    }
    let port: u16 = env::var("PORT_DEMO").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    println!("port : {port}");
    let nb = env::vars().filter(|(k, _)| k.starts_with("LC_")).count();
    println!("{nb} variable(s) LC_*");
    Ok(())
}
```

```
$ cargo run -q
erreurs : 10
/nulle/part.log : absent (No such file or directory (os error 2))
/etc/sudoers : interdit (Permission denied (os error 13))
DSN_DEMO : environment variable not found
port : 8080
2 variable(s) LC_*
$ PORT_DEMO=9000 DSN_DEMO=postgres://db1 cargo run -q | tail -3
DSN_DEMO = postgres://db1
port : 9000
2 variable(s) LC_*
```

`lines()` vient du trait `BufRead` (d'où le `use`) et rend des `io::Result<String>` sans le `\n` final : une ligne qui n'est pas de l'UTF-8 est une erreur, pas une ligne tronquée. Pour aller plus vite sur un très gros fichier, `read_line(&mut tampon)` réutilise la même `String` au lieu d'en allouer une par ligne. `BufRead` est un trait : une fonction qui prend `impl BufRead` accepte un fichier dans un `BufReader`, `io::stdin().lock()`, ou un `&[u8]` dans un test. Le labo 12 s'en sert pour tester son analyseur de journaux sans créer de fichier.

**Piège :** `File::create` sans `BufWriter` fait un appel système par `writeln!`. Sur dix mille lignes, c'est dix mille `write(2)` au lieu de trois. Et le `Drop` d'un `BufWriter` vide le tampon mais **ignore** l'erreur s'il y en a une (disque plein) : appelle `flush()` toi-même avant de considérer le fichier comme écrit. `io::stdout()` est bufferisé par ligne ; pour écrire beaucoup de lignes, prends `BufWriter::new(io::stdout().lock())`.

`io::Error` a un `kind()` qui rend un `ErrorKind` : `NotFound`, `PermissionDenied`, `AlreadyExists`, `TimedOut`, `ConnectionRefused`, `Interrupted`, et une vingtaine d'autres. C'est sur lui qu'on fait un `match`, jamais sur le texte du message, qui dépend du système (`os error 2` ici, un texte traduit ailleurs). `raw_os_error()` donne le `errno` si tu en as besoin.

### 12.4 Les variables d'environnement

`env::var("NOM")` rend `Result<String, VarError>` : `Err(NotPresent)` si elle n'existe pas, `Err(NotUnicode)` si sa valeur n'est pas de l'UTF-8. Le motif pour une valeur facultative avec défaut est la ligne du `port` ci-dessus : `.ok()` pour passer en `Option`, `.and_then(|p| p.parse().ok())` pour la convertir, `.unwrap_or(8080)`. Pour une valeur obligatoire, `env::var("DSN").context("DSN manquant")?` avec anyhow. `env::vars()` itère sur toutes les paires, et panique si l'une n'est pas de l'UTF-8 ; `env::vars_os()` ne panique jamais.

**Piège :** `env::set_var` et `env::remove_var` sont `unsafe` depuis l'édition 2024. Ce n'est pas du zèle : sous Linux, la libc lit l'environnement sans verrou, et un `getenv` dans un autre thread (une résolution DNS, une bibliothèque C) pendant un `set_var` peut lire de la mémoire libérée. Configure l'environnement d'un **processus enfant** avec `Command::env` (12.5), et passe la configuration de ton propre programme en paramètres plutôt qu'en variables globales. Dans les tests, c'est pareil : le labo 12 teste sa fonction de filtrage avec une fausse liste au lieu de modifier l'environnement du processus de test.

### 12.5 `std::process::Command`

`Command` lance un programme **sans passer par un shell** : pas d'expansion de `*`, pas de `$VAR`, pas de `;`. Chaque argument est passé tel quel, ce qui supprime d'un coup l'injection de commande. Trois façons de le lancer :

```rust
use std::io::Write;
use std::process::{Command, ExitCode, Stdio};

fn main() -> std::io::Result<ExitCode> {
    // output() : lance, attend, capture stdout et stderr.
    let sortie = Command::new("uname").arg("-sm").output()?;
    println!("uname : {}", String::from_utf8_lossy(&sortie.stdout).trim());

    // status() : lance, attend, hérite du terminal ; on ne garde que le code.
    let st = Command::new("ls").arg("/nulle/part").stderr(Stdio::null()).status()?;
    println!("ls /nulle/part : succès {}, code {:?}", st.success(), st.code());

    // spawn() : lance sans attendre ; stdin et stdout en tuyau, comme `| sort |`.
    let mut tri = Command::new("sort").arg("-r").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
    {
        let mut entree = tri.stdin.take().expect("stdin en tuyau");
        entree.write_all(b"db1\nweb2\ncache\n")?;
    } // entree est fermée ici : sort voit la fin de fichier et peut répondre
    let resultat = tri.wait_with_output()?;
    print!("sort -r :\n{}", String::from_utf8_lossy(&resultat.stdout));

    // Une commande introuvable est une io::Error, pas un code de sortie.
    if let Err(e) = Command::new("commande-absente").status() {
        eprintln!("commande-absente : {e}");
    }
    Ok(if st.success() { ExitCode::SUCCESS } else { ExitCode::from(3) })
}
```

```
$ cargo run -q ; echo "code $?"
uname : Darwin arm64
ls /nulle/part : succès false, code Some(1)
sort -r :
web2
db1
cache
commande-absente : No such file or directory (os error 2)
code 3
```

- `output()` attend la fin et rend un `Output` avec `status`, `stdout` et `stderr` en **octets** (`Vec<u8>`) : `String::from_utf8_lossy` pour les lire. Par défaut, stdin est fermé.
- `status()` laisse l'enfant écrire directement sur ton terminal. C'est le `system()` du C, sans shell.
- `spawn()` rend un `Child` tout de suite. Avec `Stdio::piped()`, on écrit dans son `stdin` et on lit son `stdout`. **Piège** : tant que tu tiens le `stdin` de l'enfant, il attend la suite ; le bloc qui le détruit (ou un `drop`) est ce qui lui dit « c'est fini ». Et si tu écris beaucoup sans lire en même temps, les deux tuyaux se remplissent et tout se bloque : écris depuis un thread, ou utilise `wait_with_output` qui lit pendant que l'enfant tourne.
- `status.code()` rend `Option<i32>` : `None` quand un signal a tué le processus (Unix). `status.success()` est vrai pour le code 0.
- Une commande **introuvable** est une `Err` de `output()` ou `status()`, pas un code 127 comme dans le shell.

Les autres réglages du constructeur : `.args([...])`, `.env("PGPASSWORD", ...)`, `.env_clear()`, `.current_dir(...)`. Pour un vrai pipeline shell (`a | b | c`), soit tu branches les `Stdio` à la main, soit tu lances `sh -c "..."` en connaissance de cause, en ne mettant jamais de donnée extérieure dans la chaîne.

**Venant de Go :** `exec.Command(...).Output()`, `.Run()` et `.Start()` sont `output()`, `status()` et `spawn()`. La différence notable : `cmd.Output()` en Go rend une erreur pour un code non nul (`*exec.ExitError`), alors qu'en Rust un code non nul est un `Ok` dont le `status` n'est pas `success()`. C'est plus juste : « la commande a dit non » n'est pas « la commande n'a pas pu tourner ».

**Venant de Python :** `subprocess.run(["ls", "-l"], capture_output=True)` est `Command::new("ls").arg("-l").output()`, et `check=True` est un `if !status.success()` que tu écris. Et comme `subprocess` sans `shell=True`, il n'y a pas de shell.

**Venant du C :** `fork` plus `execvp` plus `waitpid`, avec les tuyaux montés pour toi. `popen` passe par `/bin/sh` ; `Command` non.

### 12.6 `ExitCode`, stdout et stderr

Un outil en ligne de commande a deux sorties et un code. **stdout** porte les données, celles qu'un autre programme lira (`dsk gros . --json | jq`). **stderr** porte tout le reste : les erreurs, les avertissements, les bilans, la progression. Si tu mélanges, le JSON que reçoit `jq` contient une ligne « 3 fichiers trouvés » et casse. `println!` écrit sur stdout, `eprintln!` sur stderr.

Le **code de sortie** est ce que lisent `&&`, `set -e`, systemd et ta CI. La convention : 0 pour le succès, 1 pour un échec, 2 pour une erreur d'usage (une option inconnue, un argument manquant). `main` peut rendre un `ExitCode` (`ExitCode::SUCCESS`, `ExitCode::FAILURE`, `ExitCode::from(2)`) : c'est la façon propre, parce que les `Drop` s'exécutent et vident les tampons. `std::process::exit(n)` sort immédiatement, **sans** exécuter les destructeurs : un `BufWriter` non vidé perd ses données. `main` peut aussi rendre `anyhow::Result<()>` : en cas d'erreur, le message est écrit sur stderr avec `Error: ` devant, et le code est 1. Pour un outil, préfère attraper l'erreur dans `main` et l'écrire toi-même, avec le nom de l'outil devant, comme le fait le labo 12.

### 12.7 `clap` en derive

Écrire un analyseur d'arguments à la main, c'est ce que fait le `main.rs` du labo 11 : quarante lignes pour deux options, sans aide, sans `--version`, sans suggestion sur une faute de frappe. `clap` le fait à partir d'une struct annotée :

```rust
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Sauvegarde des bases PostgreSQL (exemple du chapitre 12).
#[derive(Debug, Parser)]
#[command(name = "pgsave", version, about)]
struct Cli {
    /// Plus de détails (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbeux: u8,

    #[command(subcommand)]
    commande: Commande,
}

#[derive(Debug, Subcommand)]
enum Commande {
    /// Sauvegarde une base
    Sauver(Sauver),
    /// Liste les sauvegardes d'un dossier
    Lister {
        /// Le dossier des sauvegardes
        #[arg(default_value = "/var/backups/pg")]
        dossier: PathBuf,
    },
}

#[derive(Debug, Args)]
struct Sauver {
    /// La base à sauvegarder
    base: String,
    /// Chaîne de connexion
    #[arg(long, env = "PGSAVE_DSN", default_value = "postgres://localhost")]
    dsn: String,
    /// Nombre de sauvegardes à garder
    #[arg(short = 'n', long, default_value_t = 7)]
    garder: u32,
    /// Compresser avec zstd
    #[arg(long)]
    compresser: bool,
}

fn main() {
    let cli = Cli::parse();
    match cli.commande {
        Commande::Sauver(s) => println!(
            "sauver {} depuis {} (garder {}, zstd {}, verbosité {})",
            s.base, s.dsn, s.garder, s.compresser, cli.verbeux
        ),
        Commande::Lister { dossier } => println!("lister {}", dossier.display()),
    }
}
```

Dans le `Cargo.toml` : `clap = { version = "4", features = ["derive", "env"] }`. Ce qu'on obtient :

```
$ pgsave --help
Sauvegarde des bases PostgreSQL (exemple du chapitre 12)

Usage: pgsave [OPTIONS] <COMMAND>

Commands:
  sauver  Sauvegarde une base
  lister  Liste les sauvegardes d'un dossier
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbeux...  Plus de détails (-v, -vv)
  -h, --help        Print help
  -V, --version     Print version
$ pgsave sauver --help
Sauvegarde une base

Usage: pgsave sauver [OPTIONS] <BASE>

Arguments:
  <BASE>  La base à sauvegarder

Options:
      --dsn <DSN>        Chaîne de connexion [env: PGSAVE_DSN=] [default: postgres://localhost]
  -v, --verbeux...       Plus de détails (-v, -vv)
  -n, --garder <GARDER>  Nombre de sauvegardes à garder [default: 7]
      --compresser       Compresser avec zstd
  -h, --help             Print help
$ pgsave sauver app -n 3 --compresser -vv
sauver app depuis postgres://localhost (garder 3, zstd true, verbosité 2)
$ PGSAVE_DSN=postgres://db1 pgsave sauver app
sauver app depuis postgres://db1 (garder 7, zstd false, verbosité 0)
$ pgsave --version
pgsave 0.1.0
```

Les règles de lecture du derive, toutes visibles ci-dessus :

- `Parser` sur la struct racine, `Subcommand` sur un enum (une variante par sous-commande, en minuscules dans la ligne de commande), `Args` sur une struct d'options réutilisable (`Sauver(Sauver)`, ou `#[command(flatten)]` pour l'inclure dans une autre).
- Un champ sans attribut est un **argument positionnel** (`base`). `#[arg(short, long)]` en fait une option `-g`/`--garder`, la lettre et le nom étant tirés du champ (`short = 'n'` pour choisir).
- Le **type du champ** décide du reste : `String` obligatoire, `Option<String>` facultatif, `bool` un drapeau, `Vec<String>` répétable, `u32` converti et vérifié, `PathBuf` un chemin. N'importe quel type qui implémente `FromStr` marche, le tien compris (le `Niveau` du labo 12).
- `default_value_t = 7` donne un défaut typé, `default_value = "..."` un défaut en texte, `env = "PGSAVE_DSN"` lit une variable d'environnement quand l'option est absente (feature `env`). L'ordre de priorité est ligne de commande, puis variable, puis défaut, et l'aide affiche les trois.
- Les commentaires `///` deviennent l'aide. `version` prend la version du `Cargo.toml`, `about` le commentaire de la struct.

Et les erreurs :

```
$ pgsave sauver ; echo "code $?"
error: the following required arguments were not provided:
  <BASE>

Usage: pgsave sauver <BASE>

For more information, try '--help'.
code 2
$ pgsave sauver app -n trois ; echo "code $?"
error: invalid value 'trois' for '--garder <GARDER>': invalid digit found in string

For more information, try '--help'.
code 2
$ pgsave sauvr app ; echo "code $?"
error: unrecognized subcommand 'sauvr'

  tip: a similar subcommand exists: 'sauver'

Usage: pgsave [OPTIONS] <COMMAND>

For more information, try '--help'.
code 2
```

Code 2, sur stderr, avec l'usage et une suggestion, sans une ligne de ton code. `Cli::parse()` sort du programme sur une erreur d'usage ; `Cli::try_parse_from(["pgsave", "sauver"])` rend un `Result` à la place, ce qui permet de **tester la CLI** sans lancer de processus, comme le font les tests de `main.rs` du labo 12. Ajoute aussi un test qui appelle `Cli::command().debug_assert()` (trait `CommandFactory`) : il vérifie la définition elle-même (deux options avec la même lettre, un défaut invalide).

**Venant de Go :** `flag` est minimaliste et ne connaît ni les sous-commandes, ni les options longues à deux tirets, ni les variables d'environnement ; on finit par prendre `cobra`. `clap` est `cobra` déclaratif : la struct **est** la définition, et le résultat est typé, sans `flag.String` qui rend un pointeur.

**Venant de Python :** c'est `argparse` (ou `typer`, qui fait la même chose à partir des annotations de type), avec la vérification à la compilation : un champ renommé dans la struct est renommé partout.

**Venant du C :** `getopt_long`, sa boucle et son `switch`, remplacés par une struct.

### 12.8 `serde` : Serialize et Deserialize

`serde` est un cadre de (dé)sérialisation : deux traits, `Serialize` et `Deserialize`, et des macros `derive` qui les implémentent pour tes types. Le format est une autre crate : `serde_json`, `toml`, `serde_yaml`, `csv`, `bincode`. Tes types ne connaissent pas le format, et le même `#[derive]` sert pour tous.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Etat {
    EnLigne,
    HorsLigne,
    EnMaintenance,
}

#[derive(Debug, Serialize, Deserialize)]
struct Serveur {
    nom: String,
    port: u16,
    etat: Etat,
    // Absent du JSON : None. None à l'écriture : champ omis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commentaire: Option<String>,
    // Absent du JSON : la valeur par défaut du type (un Vec vide).
    #[serde(default)]
    etiquettes: Vec<String>,
    // Le nom Rust et le nom JSON diffèrent : rename.
    #[serde(rename = "maxConnexions")]
    max_connexions: u32,
}

fn main() -> anyhow::Result<()> {
    let s = Serveur {
        nom: "db1".into(),
        port: 5432,
        etat: Etat::EnMaintenance,
        commentaire: None,
        etiquettes: vec!["pg17".into()],
        max_connexions: 200,
    };
    println!("{}", serde_json::to_string_pretty(&s)?);

    // Un champ inconnu (« zone ») est ignoré, les absents prennent leur défaut.
    let lu: Serveur = serde_json::from_str(
        r#"{"nom":"web1","port":443,"etat":"en_ligne","maxConnexions":50,"zone":"eu"}"#,
    )?;
    println!("{lu:?}");

    // Une faute de type : l'erreur dit où.
    let e = serde_json::from_str::<Serveur>(r#"{"nom":"web2","port":"https","etat":"en_ligne","maxConnexions":1}"#)
        .unwrap_err();
    println!("erreur : {e}");
    let e = serde_json::from_str::<Serveur>(r#"{"nom":"web3","port":80,"etat":"allume","maxConnexions":1}"#)
        .unwrap_err();
    println!("erreur : {e}");
    Ok(())
}
```

```
$ cargo run -q
{
  "nom": "db1",
  "port": 5432,
  "etat": "en_maintenance",
  "etiquettes": [
    "pg17"
  ],
  "maxConnexions": 200
}
Serveur { nom: "web1", port: 443, etat: EnLigne, commentaire: None, etiquettes: [], max_connexions: 50 }
erreur : invalid type: string "https", expected u16 at line 1 column 28
erreur : unknown variant `allume`, expected one of `en_ligne`, `hors_ligne`, `en_maintenance` at line 1 column 39
```

Les attributs à connaître, du plus utile au moins utile :

- `#[serde(rename_all = "snake_case")]` sur un type renomme tous ses champs ou variantes : `EnMaintenance` devient `en_maintenance`. Les autres valeurs : `"camelCase"`, `"kebab-case"`, `"UPPERCASE"` (le labo 12 l'utilise pour écrire `"ERROR"`), `"SCREAMING_SNAKE_CASE"`. `#[serde(rename = "...")]` sur un seul champ.
- **`Option<T>`** pour un champ qui peut manquer : absent du JSON, il vaut `None`. Avec `skip_serializing_if = "Option::is_none"`, un `None` n'est pas écrit du tout au lieu d'écrire `null`.
- `#[serde(default)]` : un champ absent prend `Default::default()` (un `Vec` vide, `0`, `false`), ou `default = "fonction"` pour une valeur à toi. Sur la struct entière, tous les champs absents prennent le défaut de la struct.
- `#[serde(flatten)]` pour inclure les champs d'une sous-struct au même niveau, `#[serde(skip)]` pour un champ jamais (dé)sérialisé, `#[serde(tag = "type")]` sur un enum pour la forme `{"type": "tcp", ...}`.

Les erreurs de lecture disent ce qui était attendu, ce qui a été trouvé, la ligne et la colonne. C'est un gros avantage sur un `json.loads` suivi de `d["port"]` : la validation est faite **à la lecture**, par le type, et une fois le `Serveur` construit, le port est un `u16`, point.

**Venant de Go :** `serde` remplace `encoding/json` et ses tags. `` `json:"maxConnexions,omitempty"` `` devient `#[serde(rename = "maxConnexions", skip_serializing_if = "...")]`, et `rename_all` évite d'écrire un tag par champ. Comme en Go, **les champs inconnus sont ignorés par défaut** (`zone` ci-dessus) ; `#[serde(deny_unknown_fields)]` est le `DisallowUnknownFields()` du décodeur Go. Deux différences qui comptent. Un champ absent n'est pas silencieusement mis à zéro comme en Go : sans `Option` ni `default`, c'est une erreur (« missing field `port` »), ce qui distingue « port 0 » de « port oublié ». Et il n'y a pas de réflexion à l'exécution : le code de (dé)sérialisation est généré à la compilation, et il est rapide.

### 12.9 `serde_json` : `Value`, `json!`, `from_reader`

Trois outils de plus, pour le JSON dont on ne connaît pas la forme, celui qu'on fabrique à la volée, et celui qu'on lit dans un fichier :

```rust
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    port: u16,
    #[serde(default)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    // deny_unknown_fields : une faute de frappe dans une clé est une erreur.
    let e = serde_json::from_str::<Config>(r#"{"port": 8080, "debgu": true}"#).unwrap_err();
    println!("config : {e}");

    // Value : du JSON dont on ne connaît pas la forme (la réponse d'une API).
    let v: Value = serde_json::from_str(
        r#"{"status":"success","data":{"result":[{"metric":{"instance":"db1:9187"},"value":[1761350400,"0.93"]}]}}"#,
    )?;
    let premier = &v["data"]["result"][0];
    println!("instance : {}", premier["metric"]["instance"]);
    println!("valeur   : {:?}", premier["value"][1].as_str());
    println!("absent   : {}", v["data"]["nulle"]["part"]); // Null, pas de panique
    if let Some(liste) = v["data"]["result"].as_array() {
        println!("{} série(s)", liste.len());
    }

    // json! construit une Value à la volée, pratique pour une réponse ponctuelle.
    let rapport = json!({ "hote": "db1", "ok": true, "latences_ms": [1.2, 0.9] });
    println!("{rapport}");

    // from_reader lit directement depuis un fichier (ou un socket, ou stdin).
    let chemin = std::env::temp_dir().join(format!("demo-{}.json", std::process::id()));
    std::fs::write(&chemin, r#"{"port": 9090}"#)?;
    let c: Config = serde_json::from_reader(BufReader::new(File::open(&chemin)?))?;
    println!("port {}, debug {}", c.port, c.debug);
    std::fs::remove_file(&chemin)?;
    Ok(())
}
```

```
$ cargo run -q
config : unknown field `debgu`, expected `port` or `debug` at line 1 column 22
instance : "db1:9187"
valeur   : Some("0.93")
absent   : null
1 série(s)
{"hote":"db1","latences_ms":[1.2,0.9],"ok":true}
port 9090, debug false
```

- `deny_unknown_fields` sur une configuration écrite à la main, toujours : une clé mal orthographiée doit être une erreur, pas une option silencieusement ignorée. Sur la réponse d'une API que tu ne contrôles pas, jamais : elle ajoutera des champs demain.
- `Value` est un enum (`Null`, `Bool`, `Number`, `String`, `Array`, `Object`). L'indexation `v["a"][0]` ne panique pas : une clé ou un indice absent rend `Value::Null`. `as_str()`, `as_u64()`, `as_array()` rendent des `Option`. C'est le bon outil pour fouiller trois niveaux dans une réponse de Prometheus ; dès que tu lis plus de deux champs, écris une struct avec seulement ces champs-là, serde ignorera le reste.
- `json!` construit une `Value` avec la syntaxe du JSON. **Piège** : les clés d'un objet `Value` sont triées (`ok` est passé à la fin), parce que la `Map` de serde_json est un `BTreeMap` par défaut. Une struct garde l'ordre de ses champs ; pour garder l'ordre d'insertion d'une `Value`, la feature `preserve_order` de serde_json.
- `from_reader` lit depuis n'importe quel `io::Read`. Mets un `BufReader` devant un fichier : `from_reader` lit octet par octet, et sans tampon c'est un appel système par octet. Pour un fichier de configuration, `from_str(&fs::read_to_string(p)?)` est aussi bien et souvent plus rapide.

**toml et serde_yaml.** Pour une configuration, TOML est le format de l'écosystème (c'est celui de `Cargo.toml`) : `cargo add toml`, puis `let c: Config = toml::from_str(&texte)?;`, avec **les mêmes** `#[derive(Deserialize)]` et les mêmes attributs. YAML (pour lire des manifestes Kubernetes ou un inventaire Ansible) suit le même principe : `serde_yaml` a longtemps été la crate de référence, elle a été archivée par son auteur en 2024, et des forks ont pris la suite ; regarde sur blessed.rs (chapitre 9) lequel est recommandé quand tu lis ces lignes. C'est la force de serde : changer de format, c'est changer une ligne.

**Venant de Python :** `json.loads` rend des `dict` sans schéma ; `Value` est son équivalent, et la struct dérivée est ce que `pydantic` apporte à Python : la validation à la lecture, par les types.

### 12.10 Des tests avec de vrais fichiers

Un outil qui manipule des fichiers se teste avec de vrais fichiers, dans un dossier temporaire **unique** (les tests tournent en parallèle, chapitre 9) et **nettoyé** même si le test échoue. La crate `tempfile` (`tempfile::tempdir()`) le fait très bien, en dépendance de développement. Sans dépendance, vingt lignes suffisent : un dossier sous `std::env::temp_dir()` dont le nom contient le pid et un compteur, et un type dont le `Drop` le supprime. C'est le `DossierTemp` des tests du labo 12 :

```
struct DossierTemp(PathBuf);
impl Drop for DossierTemp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
```

Le `Drop` s'exécute aussi quand le test panique sur un `assert!` : la pile est déroulée, les destructeurs sont appelés. C'est l'équivalent du `t.TempDir()` de Go et du `tmp_path` de pytest. Pour les fonctions qui lisent, préfère encore mieux : fais-leur prendre un `impl BufRead` ou un `impl Read`, et passe-leur un `&[u8]` dans le test. Pas de fichier, pas de nettoyage.

### 12.11 Pour le labo

Le [labo 12](../labs/12-cli-devops/README.md) assemble tout le chapitre dans `dsk`, un outil à quatre sous-commandes : `gros` (les plus gros fichiers d'un dossier, parcours récursif, taille lisible ou JSON), `logs` (filtrer un journal par niveau, des `Entree` sérialisées en JSON, un comptage par niveau), `env` (les variables d'environnement d'un préfixe, avec une option qui lit elle-même une variable d'environnement) et `run` (lancer une commande, la chronométrer, capturer ses sorties, rendre un rapport JSON et sortir avec son code). Toute la logique est dans `lib.rs`, testée avec des fichiers dans un dossier temporaire nettoyé, et la CLI clap est testée par `try_parse_from`.

### À retenir

- `fs::read_to_string`, `fs::write`, `create_dir_all`, `read_dir` (chaque entrée est un `Result`), `metadata`, `remove_dir_all` ; `DirEntry::file_type` ne suit pas les liens.
- `Path` et `PathBuf` plutôt que des chaînes ; `.display()` pour afficher ; `join` d'un chemin absolu remplace tout, vérifie ce qui vient de l'extérieur.
- `BufReader` et `lines()` pour lire en flux, `BufWriter` et un `flush()` explicite pour écrire ; `impl BufRead` rend une fonction testable avec un `&[u8]`.
- `io::Error::kind()` (`NotFound`, `PermissionDenied`) pour décider, jamais le texte ; ajoute le chemin au message avec `with_context`.
- `env::var` rend un `Result` ; `set_var` est `unsafe` depuis l'édition 2024, configure l'environnement des enfants avec `Command::env`.
- `Command` ne passe pas par un shell ; `output()` capture, `status()` hérite, `spawn()` avec `Stdio::piped()` ; un code non nul n'est pas une `Err`, une commande introuvable si.
- Données sur stdout, tout le reste sur stderr ; `main` rend un `ExitCode` (0, 1, 2) ; `process::exit` saute les `Drop`.
- clap en derive : la struct est la définition, le type du champ décide, `default_value_t` et `env` pour les défauts, l'aide vient des `///`, les erreurs d'usage sortent en code 2, `try_parse_from` pour tester.
- serde : `derive(Serialize, Deserialize)`, `rename_all`, `Option` et `default` pour les absents, `skip_serializing_if`, champs inconnus ignorés sauf `deny_unknown_fields` ; `Value` et `json!` pour le JSON sans schéma ; le même derive pour TOML et YAML.

---

← [11. Concurrence : threads, channels, Arc et Mutex, Send et Sync](11-concurrence.md) · [Sommaire](../README.md) · [13. HTTP et async : tokio, reqwest, axum](13-http-et-async.md) →
