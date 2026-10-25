// Labo 05, solution : le programme qui passe un journal à la boîte à outils.
// Lancé par : (cd solution && cargo run)   ou   (cd solution && cargo run -- ../exemple.log)
//
// main lit le texte (un fichier passé en argument, ou un journal intégré),
// appelle les fonctions de la bibliothèque et affiche. Aucune logique ici :
// tout ce qui mérite un test est dans lib.rs.

use std::env;
use std::fs;
use std::process;

use outils::{
    LigneLog, compter_mots, compter_par_niveau, dedupliquer, grouper_par_extension,
    parser_ligne_log, statistiques, top_n,
};

// Le journal par défaut, compilé dans le binaire : include_str! lit le
// fichier à la compilation et en fait une &'static str. Le chemin est
// relatif à ce fichier source.
const JOURNAL: &str = include_str!("../../exemple.log");

fn main() {
    // args() rend un itérateur ; nth(1) saute le nom du programme. On garde
    // le texte dans une String possédée par main : toutes les tranches et les
    // LigneLog qui suivent l'empruntent ou en copient des morceaux.
    let texte = match env::args().nth(1) {
        Some(chemin) => match fs::read_to_string(&chemin) {
            Ok(contenu) => contenu,
            Err(e) => {
                // Pas encore de gestion d'erreur propre (chapitre 8) : un
                // message sur stderr et un code de sortie non nul suffisent.
                eprintln!("impossible de lire {chemin} : {e}");
                process::exit(1);
            }
        },
        None => JOURNAL.to_string(),
    };

    // filter_map avec la fonction elle-même, sans fermeture : parser_ligne_log
    // prend une &str et rend un Option, exactement ce que filter_map attend.
    let lignes: Vec<LigneLog> = texte.lines().filter_map(parser_ligne_log).collect();
    let illisibles = texte.lines().count() - lignes.len();
    println!(
        "lignes lues : {}, illisibles : {}",
        lignes.len() + illisibles,
        illisibles
    );

    // BTreeMap : l'affichage suit l'ordre des niveaux, Debug d'abord.
    for (niveau, nombre) in compter_par_niveau(&lignes) {
        println!("  {:<6} {nombre}", format!("{niveau:?}"));
    }

    // Les mots des messages, tous niveaux confondus. On recolle les messages
    // en un seul texte avec join : compter_mots attend une &str.
    let messages: Vec<&str> = lignes.iter().map(|l| l.message.as_str()).collect();
    let compte = compter_mots(&messages.join(" "));
    println!("mots les plus fréquents :");
    for (mot, nombre) in top_n(&compte, 3) {
        println!("  {mot:<10} {nombre}");
    }

    // Les erreurs distinctes, dans l'ordre où elles sont arrivées.
    let erreurs: Vec<String> = lignes
        .iter()
        .filter(|l| l.niveau == outils::Niveau::Error)
        .map(|l| l.message.clone())
        .collect();
    println!("erreurs distinctes :");
    for e in dedupliquer(erreurs) {
        println!("  {e}");
    }

    // Les latences : le mot qui précède « ms » dans chaque message. windows(2)
    // fait glisser une fenêtre sur les mots, et filter_map garde ceux qui se
    // convertissent en f64.
    let latences: Vec<f64> = lignes
        .iter()
        .flat_map(|l| {
            let mots: Vec<&str> = l.message.split_whitespace().collect();
            mots.windows(2)
                .filter(|w| w[1] == "ms")
                .filter_map(|w| w[0].parse::<f64>().ok())
                .collect::<Vec<f64>>()
        })
        .collect();
    match statistiques(&latences) {
        Some(s) => println!(
            "latences : min {} max {} moyenne {:.1} médiane {}",
            s.min, s.max, s.moyenne, s.mediane
        ),
        None => println!("latences : aucune mesure"),
    }

    // Les chemins de fichiers cités : les mots qui commencent par '/'.
    let chemins: Vec<&str> = lignes
        .iter()
        .flat_map(|l| l.message.split_whitespace())
        .filter(|mot| mot.starts_with('/'))
        .collect();
    println!("fichiers par extension :");
    for (extension, fichiers) in grouper_par_extension(&chemins) {
        // La clé vide est affichée lisiblement plutôt qu'en blanc.
        let nom = if extension.is_empty() {
            "(aucune)"
        } else {
            &extension
        };
        println!("  {nom:<8} {}", fichiers.join(", "));
    }
}
