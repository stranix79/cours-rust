// Labo 09 : la ligne de commande `semver` (fournie complète, n'y touche pas).
// Lancé par : cargo run -- compare 1.2.3 1.10.0
//
//   semver compare 1.2.3 1.10.0       affiche <, = ou >
//   semver sort 1.10.0 1.2.3 1.2.3-rc.1   affiche les versions triées, une par ligne
//   semver satisfait "^1.2" 1.5.0     affiche oui (code 0) ou non (code 1)
//
// Codes de sortie : 0 succès, 1 « non » pour satisfait, 2 erreur d'usage ou
// version illisible. Ce binaire est un CLIENT de la bibliothèque : il
// l'importe par son nom (semver_maison), exactement comme le ferait une
// autre crate. Il ne voit que ce que lib.rs exporte avec `pub use`.

use std::io::{self, Write};
use std::process::ExitCode;

use semver_maison::{Contrainte, Version};

const USAGE: &str = "usage : semver compare A B | sort V... | satisfait CONTRAINTE V";

/// Toute la logique de la CLI, testable : les arguments en tranche, la
/// sortie dans n'importe quel Write (stdout en vrai, un Vec<u8> en test).
/// Rend le code de sortie ; les erreurs vont sur stderr.
fn executer(args: &[String], sortie: &mut dyn Write) -> u8 {
    match lancer(args, sortie) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("semver : {message}");
            2
        }
    }
}

/// Le vrai travail, avec `?` partout : chaque erreur devient une String
/// (map_err + to_string), executer la transforme en code 2.
fn lancer(args: &[String], sortie: &mut dyn Write) -> Result<u8, String> {
    // Les motifs de tranche découpent les arguments et vérifient leur nombre
    // d'un seul coup : [commande, a, b] n'accepte que trois éléments.
    match args {
        [commande, a, b] if commande == "compare" => {
            let a = Version::parse(a).map_err(|e| e.to_string())?;
            let b = Version::parse(b).map_err(|e| e.to_string())?;
            // Ordering est un enum : un match exhaustif donne le symbole.
            let symbole = match a.cmp(&b) {
                std::cmp::Ordering::Less => "<",
                std::cmp::Ordering::Equal => "=",
                std::cmp::Ordering::Greater => ">",
            };
            writeln!(sortie, "{symbole}").map_err(|e| e.to_string())?;
            Ok(0)
        }
        [commande, versions @ ..] if commande == "sort" && !versions.is_empty() => {
            // collect sur des Result : le premier Err arrête tout et devient
            // le résultat ; sinon on a un Vec<Version>.
            let mut v: Vec<Version> = versions
                .iter()
                .map(|s| Version::parse(s))
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            // sort marche parce que Version implémente Ord. Tri stable : deux
            // versions égales (qui ne diffèrent que par le build) gardent leur
            // ordre d'entrée.
            v.sort();
            for x in &v {
                writeln!(sortie, "{x}").map_err(|e| e.to_string())?;
            }
            Ok(0)
        }
        [commande, contrainte, v] if commande == "satisfait" => {
            let c = Contrainte::parse(contrainte).map_err(|e| e.to_string())?;
            let v = Version::parse(v).map_err(|e| e.to_string())?;
            let oui = c.satisfait(&v);
            writeln!(sortie, "{}", if oui { "oui" } else { "non" }).map_err(|e| e.to_string())?;
            // Code 1 pour « non » : `semver satisfait ... && deploy` marche.
            Ok(if oui { 0 } else { 1 })
        }
        _ => Err(USAGE.to_string()),
    }
}

fn main() -> ExitCode {
    // skip(1) : le premier argument est le nom du programme.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = executer(&args, &mut io::stdout());
    ExitCode::from(code)
}

// Des tests unitaires DANS le binaire : cargo test les lance aussi. Ils
// appellent executer, une fonction privée de main.rs, sans lancer de
// processus.
#[cfg(test)]
mod tests {
    use super::*;

    fn lancer_cli(args: &[&str]) -> (u8, String) {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let mut sortie = Vec::new(); // Vec<u8> implémente Write
        let code = executer(&args, &mut sortie);
        (code, String::from_utf8(sortie).unwrap())
    }

    #[test]
    fn compare() {
        assert_eq!(
            lancer_cli(&["compare", "1.2.3", "1.10.0"]),
            (0, "<\n".into())
        );
        assert_eq!(
            lancer_cli(&["compare", "1.2.3", "v1.2.3"]),
            (0, "=\n".into())
        );
        assert_eq!(
            lancer_cli(&["compare", "2.0.0", "2.0.0-rc.1"]),
            (0, ">\n".into())
        );
    }

    #[test]
    fn sort() {
        let (code, texte) = lancer_cli(&["sort", "1.10.0", "1.2.3", "1.2.3-rc.1", "0.9.0"]);
        assert_eq!(code, 0);
        assert_eq!(texte, "0.9.0\n1.2.3-rc.1\n1.2.3\n1.10.0\n");
    }

    #[test]
    fn satisfait_oui_non() {
        assert_eq!(
            lancer_cli(&["satisfait", "^1.2", "1.5.0"]),
            (0, "oui\n".into())
        );
        assert_eq!(
            lancer_cli(&["satisfait", "^1.2", "2.0.0"]),
            (1, "non\n".into())
        );
    }

    #[test]
    fn erreurs_d_usage() {
        assert_eq!(lancer_cli(&[]).0, 2);
        assert_eq!(lancer_cli(&["compare", "1.0.0"]).0, 2);
        assert_eq!(lancer_cli(&["diff", "1.0.0", "2.0.0"]).0, 2);
        assert_eq!(lancer_cli(&["compare", "1.0", "2.0.0"]).0, 2);
        assert_eq!(lancer_cli(&["sort"]).0, 2);
    }
}
