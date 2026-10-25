// Labo 05, solution : une boîte à outils de sysadmin, une chaîne d'itérateurs par fonction.
// Lancé par : (cd solution && cargo test)   depuis labs/05-boite-a-outils
//
// Rôle du fichier : toute la logique testable. main.rs ne fait que lire un
// texte et appeler ces fonctions ; tests/outils.rs les vérifie une par une.
// Chaque fonction se lit de haut en bas comme une suite d'étapes : d'où
// viennent les éléments, ce qu'on garde, ce qu'on transforme, où on range.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// Compte les mots d'un texte. Un mot est une suite de caractères
/// alphanumériques (lettres accentuées comprises), mis en minuscules ;
/// tout le reste (espaces, ponctuation, tirets, apostrophes) sépare.
pub fn compter_mots(texte: &str) -> HashMap<String, usize> {
    // On part d'une map vide et on la remplit dans un fold : l'accumulateur
    // est la map elle-même, que chaque mot met à jour puis rend.
    texte
        // split avec une fermeture : on coupe sur chaque caractère qui n'est
        // pas alphanumérique. is_alphanumeric connaît l'Unicode, donc « é »
        // reste dans le mot, là où un test ASCII l'aurait coupé.
        .split(|c: char| !c.is_alphanumeric())
        // Deux séparateurs de suite (« , » puis espace) donnent un morceau
        // vide entre eux : split les garde, on les jette.
        .filter(|mot| !mot.is_empty())
        // to_lowercase alloue une String neuve : la clé de la map doit
        // posséder son texte, puisque « Erreur » et « erreur » ne sont pas
        // la même tranche du texte d'origine.
        .map(str::to_lowercase)
        .fold(HashMap::new(), |mut compte, mot| {
            // entry cherche la clé une seule fois ; or_insert(0) crée la case
            // si besoin et rend un &mut usize, qu'on déréférence pour ajouter.
            *compte.entry(mot).or_insert(0) += 1;
            compte
        })
}

/// Les `n` mots les plus fréquents, du plus fréquent au moins fréquent ; à
/// égalité, dans l'ordre alphabétique. Le résultat est donc toujours le même,
/// quel que soit l'ordre (aléatoire) de la HashMap.
pub fn top_n(compte: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    // iter() donne des (&String, &usize) : on emprunte la map, on ne la vide
    // pas. On clone le mot pour que le résultat possède ses chaînes et ne
    // dépende plus de la map de l'appelant.
    let mut paires: Vec<(String, usize)> = compte
        .iter()
        .map(|(mot, &nombre)| (mot.clone(), nombre))
        .collect();
    // Un seul tri, deux critères : d'abord le nombre décroissant (b avant a),
    // puis, si égalité, le mot croissant (a avant b). then_with n'appelle sa
    // fermeture que si le premier critère donne Equal.
    paires.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    // truncate coupe sur place ; si n dépasse la longueur, il ne fait rien.
    paires.truncate(n);
    paires
}

/// Retire les doublons en gardant la première occurrence, dans l'ordre
/// d'origine (contrairement à sort + dedup, qui change l'ordre).
pub fn dedupliquer(elements: Vec<String>) -> Vec<String> {
    // L'ensemble des éléments déjà vus. insert rend true la première fois
    // et false ensuite : c'est exactement le prédicat du filtre.
    let mut vus = HashSet::new();
    elements
        // into_iter : on consomme le vecteur reçu par valeur, chaque String
        // passe dans le résultat sans être recopiée.
        .into_iter()
        // La fermeture modifie `vus` : c'est une FnMut, ce que filter accepte.
        // On range un clone dans l'ensemble parce que l'original, lui, part
        // dans le résultat ; il ne peut pas être aux deux endroits.
        .filter(|e| vus.insert(e.clone()))
        .collect()
}

/// Résumé d'une série de mesures (des latences en millisecondes, par exemple).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub min: f64,
    pub max: f64,
    pub moyenne: f64,
    pub mediane: f64,
}

/// Min, max, moyenne et médiane ; `None` si la série est vide, parce qu'une
/// moyenne de rien n'a pas de sens (et diviserait par zéro).
pub fn statistiques(valeurs: &[f64]) -> Option<Stats> {
    // Le cas vide d'abord : après ce test, tout le reste peut supposer au
    // moins une valeur.
    if valeurs.is_empty() {
        return None;
    }
    // f64 n'a pas de min() d'itérateur (pas de Ord, à cause de NaN) : on plie
    // avec f64::min, en partant de l'infini, qui est plus grand que tout.
    let min = valeurs.iter().copied().fold(f64::INFINITY, f64::min);
    let max = valeurs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // sum::<f64>() a besoin du type en turbofish ; len() est un usize, qu'on
    // convertit avec `as` pour diviser deux f64.
    let moyenne = valeurs.iter().sum::<f64>() / valeurs.len() as f64;

    // La médiane demande une copie triée : on ne touche pas à la tranche de
    // l'appelant, qu'on n'a d'ailleurs qu'en lecture (&[f64]).
    let mut triees = valeurs.to_vec();
    // total_cmp donne l'ordre total qu'il manque aux flottants.
    triees.sort_by(f64::total_cmp);
    let milieu = triees.len() / 2;
    // Nombre pair de valeurs (is_multiple_of(2), que clippy préfère à % 2 == 0) :
    // la moyenne des deux du milieu. Impair : celle
    // du milieu. Les index sont sûrs, la tranche n'est pas vide.
    let mediane = if triees.len().is_multiple_of(2) {
        (triees[milieu - 1] + triees[milieu]) / 2.0
    } else {
        triees[milieu]
    };

    Some(Stats {
        min,
        max,
        moyenne,
        mediane,
    })
}

/// Range des chemins par extension (`"gz"`, `"conf"`...), dans l'ordre des
/// extensions ; la clé `""` regroupe les fichiers sans extension. Dans chaque
/// groupe, les chemins gardent leur ordre d'arrivée.
pub fn grouper_par_extension(chemins: &[&str]) -> BTreeMap<String, Vec<String>> {
    // BTreeMap plutôt que HashMap : l'affichage et les tests veulent un ordre
    // stable, et la BTreeMap itère dans l'ordre des clés.
    let mut groupes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for chemin in chemins {
        // Path::extension sait que « .bashrc » est un fichier caché sans
        // extension, et que l'extension de « db.sql.gz » est « gz ». Elle rend
        // un Option<&OsStr> : un nom de fichier n'est pas forcément de
        // l'UTF-8. to_str() rend un Option<&str>, unwrap_or("") le défaut.
        let extension = Path::new(chemin)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        // Le motif « grouper » du chapitre : entry, or_default (un Vec vide),
        // push. Une recherche par chemin, aucun if.
        groupes
            .entry(extension.to_string())
            .or_default()
            .push(chemin.to_string());
    }
    groupes
}

/// Le niveau d'une ligne de log. L'ordre des variantes compte : `derive(Ord)`
/// range Debug < Info < Warn < Error, dans l'ordre où elles sont écrites, et
/// c'est cet ordre que suit la BTreeMap de `compter_par_niveau`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Niveau {
    Debug,
    Info,
    Warn,
    Error,
}

/// Une ligne de log analysée.
#[derive(Debug, Clone, PartialEq)]
pub struct LigneLog {
    pub date: String,
    pub heure: String,
    pub niveau: Niveau,
    pub message: String,
}

/// Traduit le texte du niveau (en majuscules, tel qu'il apparaît dans les
/// logs) en `Niveau` ; `None` pour tout le reste.
pub fn niveau_depuis(texte: &str) -> Option<Niveau> {
    // Un match sur des &str : chaque bras compare la chaîne entière. Le bras
    // `_` attrape tout ce qui n'est pas prévu, le compilateur l'exige.
    match texte {
        "DEBUG" => Some(Niveau::Debug),
        "INFO" => Some(Niveau::Info),
        "WARN" => Some(Niveau::Warn),
        "ERROR" => Some(Niveau::Error),
        _ => None,
    }
}

/// Analyse `2026-10-25 01:02:03 ERROR message avec espaces`. `None` si un des
/// quatre morceaux manque ou si le niveau est inconnu. Le message garde ses
/// espaces intérieurs ; les blancs en fin de ligne sont retirés.
pub fn parser_ligne_log(ligne: &str) -> Option<LigneLog> {
    // splitn(4, ' ') coupe au plus trois fois : le quatrième morceau est
    // tout le reste de la ligne, espaces compris. C'est ce qui garde le
    // message entier.
    let mut morceaux = ligne.trim_end().splitn(4, ' ');
    // next() rend un Option ; le `?` sur un Option rend None tout de suite
    // s'il n'y a plus de morceau (chapitre 6 et 8). Quatre lignes, quatre
    // sorties possibles, aucune imbrication.
    let date = morceaux.next()?;
    let heure = morceaux.next()?;
    let niveau = niveau_depuis(morceaux.next()?)?;
    let message = morceaux.next()?;
    // Une ligne vide ou « 2026-10-25 » seul tombe sur un des `?` ci-dessus ;
    // il reste à refuser un message vide (« ... INFO  » avec deux espaces).
    if message.is_empty() {
        return None;
    }
    // On convertit en String à la fin seulement : la LigneLog doit posséder
    // son texte pour survivre à la ligne d'origine.
    Some(LigneLog {
        date: date.to_string(),
        heure: heure.to_string(),
        niveau,
        message: message.to_string(),
    })
}

/// Combien de lignes par niveau, dans l'ordre des niveaux. Un niveau absent
/// n'a pas de clé (pas de zéro).
pub fn compter_par_niveau(lignes: &[LigneLog]) -> BTreeMap<Niveau, usize> {
    // Même fold que compter_mots. La clé est un Niveau, qui est Copy : on la
    // copie depuis la ligne empruntée sans rien cloner. BTreeMap exige Ord
    // sur la clé, d'où le derive(Ord) sur Niveau.
    lignes.iter().fold(BTreeMap::new(), |mut compte, ligne| {
        *compte.entry(ligne.niveau).or_insert(0) += 1;
        compte
    })
}
