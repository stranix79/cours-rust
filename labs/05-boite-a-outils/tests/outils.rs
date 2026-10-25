// Labo 05 : les tests de la boîte à outils. Identiques dans le squelette et
// dans la solution : `use outils::...` désigne la bibliothèque du paquet.
// Lancé par : cargo test (ton code) ou (cd solution && cargo test).

use std::collections::{BTreeMap, HashMap};

use outils::*;

// Construit un Vec<String> à partir de littéraux, pour alléger les tests.
fn chaines(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn compter_mots_minuscules_et_ponctuation() {
    let compte = compter_mots("Erreur : disque plein. ERREUR, erreur ! Disque");
    assert_eq!(compte.len(), 3);
    assert_eq!(compte.get("erreur"), Some(&3));
    assert_eq!(compte.get("disque"), Some(&2));
    assert_eq!(compte.get("plein"), Some(&1));
    assert_eq!(
        compte.get("Erreur"),
        None,
        "les clés doivent être en minuscules"
    );
}

#[test]
fn compter_mots_accents_tirets_et_vide() {
    let compte = compter_mots("Liège l'été, web-01 et WEB-01");
    assert_eq!(compte.get("liège"), Some(&1));
    assert_eq!(compte.get("été"), Some(&1));
    assert_eq!(compte.get("l"), Some(&1));
    assert_eq!(compte.get("web"), Some(&2));
    assert_eq!(compte.get("01"), Some(&2));
    assert!(compter_mots("").is_empty());
    assert!(compter_mots(" ... ,, !").is_empty());
}

#[test]
fn top_n_tri_par_nombre_puis_par_mot() {
    let compte: HashMap<String, usize> =
        [("db", 2), ("web", 5), ("cache", 2), ("api", 2), ("dns", 1)]
            .into_iter()
            .map(|(m, n)| (m.to_string(), n))
            .collect();
    let top = top_n(&compte, 4);
    let attendu = vec![
        ("web".to_string(), 5),
        ("api".to_string(), 2),
        ("cache".to_string(), 2),
        ("db".to_string(), 2),
    ];
    assert_eq!(top, attendu);
}

#[test]
fn top_n_stable_et_bornes() {
    let compte = compter_mots("b a c b a c");
    // Trois mots à égalité : l'ordre alphabétique départage, à chaque appel.
    for _ in 0..20 {
        assert_eq!(
            top_n(&compter_mots("b a c b a c"), 2),
            vec![("a".to_string(), 2), ("b".to_string(), 2)]
        );
    }
    assert_eq!(
        top_n(&compte, 10).len(),
        3,
        "n plus grand que la map : tout"
    );
    assert!(top_n(&compte, 0).is_empty());
    assert!(top_n(&HashMap::new(), 3).is_empty());
}

#[test]
fn dedupliquer_garde_la_premiere_occurrence() {
    let entree = chaines(&["web-02", "db-01", "web-02", "web-01", "db-01", "web-02"]);
    assert_eq!(dedupliquer(entree), chaines(&["web-02", "db-01", "web-01"]));
    assert_eq!(dedupliquer(Vec::new()), Vec::<String>::new());
    assert_eq!(
        dedupliquer(chaines(&["a", "A"])),
        chaines(&["a", "A"]),
        "sensible à la casse"
    );
}

#[test]
fn statistiques_impair_et_pair() {
    let s = statistiques(&[12.0, 80.0, 11.0, 15.0, 14.0]).expect("série non vide");
    assert_eq!(s.min, 11.0);
    assert_eq!(s.max, 80.0);
    assert_eq!(s.moyenne, 26.4);
    assert_eq!(s.mediane, 14.0, "impair : la valeur du milieu après tri");

    let s = statistiques(&[4.0, 1.0, 3.0, 2.0]).expect("série non vide");
    assert_eq!(s.mediane, 2.5, "pair : moyenne des deux du milieu");
    assert_eq!(s.moyenne, 2.5);
}

#[test]
fn statistiques_une_valeur_et_vide() {
    let s = statistiques(&[-3.5]).expect("une valeur");
    assert_eq!(
        s,
        Stats {
            min: -3.5,
            max: -3.5,
            moyenne: -3.5,
            mediane: -3.5
        }
    );
    assert_eq!(statistiques(&[]), None);
}

#[test]
fn statistiques_ne_modifie_pas_l_entree() {
    let valeurs = vec![3.0, 1.0, 2.0];
    let _ = statistiques(&valeurs);
    assert_eq!(valeurs, vec![3.0, 1.0, 2.0]);
}

#[test]
fn grouper_par_extension_ordre_et_sans_extension() {
    let chemins = [
        "/etc/app.conf",
        "/var/backups/db.sql.gz",
        "Makefile",
        "/home/gilles/.bashrc",
        "/etc/nginx.conf",
        "logs/acces.log.gz",
    ];
    let groupes = grouper_par_extension(&chemins);
    let mut attendu: BTreeMap<String, Vec<String>> = BTreeMap::new();
    attendu.insert(
        String::new(),
        chaines(&["Makefile", "/home/gilles/.bashrc"]),
    );
    attendu.insert(
        "conf".to_string(),
        chaines(&["/etc/app.conf", "/etc/nginx.conf"]),
    );
    attendu.insert(
        "gz".to_string(),
        chaines(&["/var/backups/db.sql.gz", "logs/acces.log.gz"]),
    );
    assert_eq!(groupes, attendu);
    let cles: Vec<&String> = groupes.keys().collect();
    assert_eq!(cles, ["", "conf", "gz"]);
    assert!(grouper_par_extension(&[]).is_empty());
}

#[test]
fn niveau_depuis_texte() {
    assert_eq!(niveau_depuis("DEBUG"), Some(Niveau::Debug));
    assert_eq!(niveau_depuis("INFO"), Some(Niveau::Info));
    assert_eq!(niveau_depuis("WARN"), Some(Niveau::Warn));
    assert_eq!(niveau_depuis("ERROR"), Some(Niveau::Error));
    assert_eq!(niveau_depuis("error"), None);
    assert_eq!(niveau_depuis("FATAL"), None);
}

#[test]
fn parser_ligne_log_valide() {
    let l = parser_ligne_log("2026-10-25 01:02:03 ERROR connexion refusée par  db-01  ")
        .expect("ligne valide");
    assert_eq!(
        l,
        LigneLog {
            date: "2026-10-25".to_string(),
            heure: "01:02:03".to_string(),
            niveau: Niveau::Error,
            message: "connexion refusée par  db-01".to_string(),
        }
    );
    let l = parser_ligne_log("2026-10-25 23:59:59 DEBUG x").expect("message d'un mot");
    assert_eq!(l.niveau, Niveau::Debug);
    assert_eq!(l.message, "x");
}

#[test]
fn parser_ligne_log_invalide() {
    assert_eq!(parser_ligne_log(""), None);
    assert_eq!(parser_ligne_log("ligne illisible"), None);
    assert_eq!(
        parser_ligne_log("2026-10-25 01:02:03 INFO"),
        None,
        "message absent"
    );
    assert_eq!(
        parser_ligne_log("2026-10-25 01:02:03 INFO   "),
        None,
        "message vide"
    );
    assert_eq!(
        parser_ligne_log("2026-10-25 01:02:03 FATAL disque"),
        None,
        "niveau inconnu"
    );
}

#[test]
fn compter_par_niveau_dans_l_ordre() {
    let texte = "\
2026-10-25 01:00:00 ERROR a
2026-10-25 01:00:01 INFO b
ligne illisible
2026-10-25 01:00:02 ERROR c
2026-10-25 01:00:03 DEBUG d";
    let lignes: Vec<LigneLog> = texte.lines().filter_map(parser_ligne_log).collect();
    assert_eq!(lignes.len(), 4);
    let compte = compter_par_niveau(&lignes);
    let paires: Vec<(Niveau, usize)> = compte.into_iter().collect();
    assert_eq!(
        paires,
        vec![(Niveau::Debug, 1), (Niveau::Info, 1), (Niveau::Error, 2)]
    );
    assert!(compter_par_niveau(&[]).is_empty());
}

#[test]
fn niveau_est_ordonne() {
    assert!(Niveau::Debug < Niveau::Info);
    assert!(Niveau::Info < Niveau::Warn);
    assert!(Niveau::Warn < Niveau::Error);
}
