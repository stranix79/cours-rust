// Labo 12 : les tests d'intégration de dsk. Identiques dans le squelette et
// dans la solution. Les fichiers sont créés dans un dossier temporaire
// unique, supprimé à la fin de chaque test, même si le test échoue.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use dsk::{
    Entree, Fichier, Niveau, Rapport, bilan, filtrer_variables, lancer, parser_logs, plus_gros,
    taille_lisible, variables,
};

/// Un dossier temporaire à nous, supprimé quand la valeur est détruite.
/// C'est ce que fait la crate tempfile, en vingt lignes et sans dépendance.
struct DossierTemp(PathBuf);

impl DossierTemp {
    fn nouveau(nom: &str) -> DossierTemp {
        // Unique par processus (le pid) et par appel (le compteur) : les
        // tests tournent en parallèle, et deux lancements de cargo test
        // aussi, peut-être.
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let chemin = std::env::temp_dir().join(format!("dsk-{nom}-{}-{n}", std::process::id()));
        fs::create_dir_all(&chemin).expect("créer le dossier temporaire");
        DossierTemp(chemin)
    }

    fn ecrire(&self, relatif: &str, taille: usize) -> PathBuf {
        let chemin = self.0.join(relatif);
        fs::create_dir_all(chemin.parent().unwrap()).unwrap();
        fs::write(&chemin, vec![b'x'; taille]).unwrap();
        chemin
    }

    fn chemin(&self) -> &Path {
        &self.0
    }
}

impl Drop for DossierTemp {
    fn drop(&mut self) {
        // Drop ne peut pas échouer : on ignore l'erreur de nettoyage.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn s(mots: &[&str]) -> Vec<String> {
    mots.iter().map(|m| m.to_string()).collect()
}

#[test]
fn taille_lisible_en_puissances_de_1024() {
    assert_eq!(taille_lisible(0), "0 o");
    assert_eq!(taille_lisible(1023), "1023 o");
    assert_eq!(taille_lisible(1024), "1.0 Kio");
    assert_eq!(taille_lisible(1536), "1.5 Kio");
    assert_eq!(taille_lisible(5 * 1024 * 1024), "5.0 Mio");
    assert_eq!(taille_lisible(3 * 1024 * 1024 * 1024), "3.0 Gio");
}

#[test]
fn plus_gros_parcourt_les_sous_dossiers() {
    let d = DossierTemp::nouveau("gros");
    d.ecrire("petit.txt", 10);
    let moyen = d.ecrire("a/moyen.bin", 2000);
    let gros = d.ecrire("a/b/c/gros.bin", 5000);
    d.ecrire("vide.txt", 0);
    fs::create_dir_all(d.chemin().join("dossier-vide")).unwrap();

    let top = plus_gros(d.chemin(), 2).unwrap();
    assert_eq!(
        top,
        vec![
            Fichier {
                chemin: gros,
                taille: 5000
            },
            Fichier {
                chemin: moyen,
                taille: 2000
            },
        ]
    );
    // Plus qu'il n'y en a : tous les fichiers, pas les dossiers.
    assert_eq!(plus_gros(d.chemin(), 100).unwrap().len(), 4);
    assert!(plus_gros(d.chemin(), 0).unwrap().is_empty());
}

#[test]
fn plus_gros_a_egalite_trie_par_chemin() {
    let d = DossierTemp::nouveau("egalite");
    d.ecrire("b.txt", 100);
    d.ecrire("a.txt", 100);
    let noms: Vec<String> = plus_gros(d.chemin(), 10)
        .unwrap()
        .iter()
        .map(|f| f.chemin.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(noms, ["a.txt", "b.txt"]);
}

#[test]
fn plus_gros_dossier_absent_dit_lequel() {
    let e = plus_gros(Path::new("/nulle/part/dsk"), 5).unwrap_err();
    let texte = format!("{e:#}");
    assert!(texte.contains("/nulle/part/dsk"), "message : {texte}");
    // La cause d'origine est une io::Error NotFound, retrouvable par
    // downcast : le contexte d'anyhow ne l'a pas effacée.
    let io = e.downcast_ref::<std::io::Error>().expect("cause io::Error");
    assert_eq!(io.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn niveau_se_lit_et_s_ordonne() {
    assert_eq!("error".parse::<Niveau>(), Ok(Niveau::Error));
    assert_eq!("WARNING".parse::<Niveau>(), Ok(Niveau::Warn));
    assert!("PANIQUE".parse::<Niveau>().is_err());
    assert!(Niveau::Debug < Niveau::Info && Niveau::Warn < Niveau::Error);
    assert_eq!(Niveau::Warn.to_string(), "WARN");
    assert_eq!(
        format!("{:<5}|", Niveau::Info),
        "INFO |",
        "Display respecte la largeur"
    );
}

const JOURNAL: &str = "\
2026-10-25 01:02:03 INFO démarrage du sondeur
2026-10-25 01:02:04 WARN web1 lent (1832 ms)
ligne de pile sans horodatage
2026-10-25 01:02:05 ERROR db1:5432 connexion refusée
2026-10-25 01:02:06 DEBUG détail
2026-10-25 01:02:07 error minuscules acceptées
";

#[test]
fn parser_logs_filtre_par_niveau() {
    // &[u8] implémente BufRead : pas besoin de fichier pour tester.
    let erreurs = parser_logs(JOURNAL.as_bytes(), Niveau::Error);
    assert_eq!(
        erreurs[0],
        Entree {
            horodatage: "2026-10-25 01:02:05".to_string(),
            niveau: Niveau::Error,
            message: "db1:5432 connexion refusée".to_string(),
        }
    );
    assert_eq!(erreurs.len(), 2);
    assert_eq!(parser_logs(JOURNAL.as_bytes(), Niveau::Warn).len(), 3);
    assert_eq!(
        parser_logs(JOURNAL.as_bytes(), Niveau::Debug).len(),
        5,
        "la ligne mal formée est ignorée"
    );
}

#[test]
fn parser_logs_depuis_un_fichier() {
    let d = DossierTemp::nouveau("logs");
    let chemin = d.chemin().join("app.log");
    fs::write(&chemin, JOURNAL).unwrap();
    let f = std::io::BufReader::new(fs::File::open(&chemin).unwrap());
    let entrees = parser_logs(f, Niveau::Info);
    let compte = bilan(&entrees);
    assert_eq!(compte.get(&Niveau::Error), Some(&2));
    assert_eq!(compte.get(&Niveau::Warn), Some(&1));
    assert_eq!(compte.get(&Niveau::Info), Some(&1));
    assert_eq!(compte.get(&Niveau::Debug), None, "filtré par Info");
}

#[test]
fn entree_en_json() {
    let e = Entree {
        horodatage: "2026-10-25 01:02:03".to_string(),
        niveau: Niveau::Error,
        message: "disque plein".to_string(),
    };
    let json = serde_json::to_string(&e).unwrap();
    assert_eq!(
        json,
        r#"{"horodatage":"2026-10-25 01:02:03","niveau":"ERROR","message":"disque plein"}"#
    );
    // Aller-retour : ce qu'on écrit se relit.
    let relu: Entree = serde_json::from_str(&json).unwrap();
    assert_eq!(relu, e);
}

#[test]
fn filtrer_variables_garde_le_prefixe_et_trie() {
    let fausses = vec![
        ("DSK_ZONE".to_string(), "eu".to_string()),
        ("HOME".to_string(), "/root".to_string()),
        ("DSK_APP".to_string(), "dsk".to_string()),
        ("XDSK_NON".to_string(), "x".to_string()),
    ];
    assert_eq!(
        filtrer_variables(fausses, "DSK_"),
        vec![
            ("DSK_APP".to_string(), "dsk".to_string()),
            ("DSK_ZONE".to_string(), "eu".to_string()),
        ]
    );
}

#[test]
fn variables_lit_le_vrai_environnement() {
    // cargo test définit CARGO_PKG_NAME, CARGO_PKG_VERSION... pour le
    // processus de test : on s'en sert au lieu de modifier l'environnement
    // (std::env::set_var est unsafe depuis l'édition 2024).
    let vars = variables("CARGO_PKG_");
    assert!(
        vars.iter().any(|(nom, _)| nom == "CARGO_PKG_NAME"),
        "obtenu {vars:?}"
    );
    assert!(vars.windows(2).all(|p| p[0].0 <= p[1].0), "triées par nom");
    assert!(vars.iter().all(|(nom, _)| nom.starts_with("CARGO_PKG_")));
}

#[test]
fn lancer_rend_le_code_de_sortie() {
    let r = lancer(&s(&["sh", "-c", "exit 3"])).unwrap();
    assert_eq!(
        r.code,
        Some(3),
        "un code non nul n'est pas une erreur de lancer"
    );
    assert_eq!(r.commande, "sh -c exit 3");
}

#[test]
fn lancer_capture_stdout_et_stderr() {
    let r = lancer(&s(&["sh", "-c", "echo bonjour; echo oups >&2"])).unwrap();
    assert_eq!(r.code, Some(0));
    assert_eq!(r.stdout, "bonjour\n");
    assert_eq!(r.stderr, "oups\n");
    assert!(r.duree_ms < 5000, "durée : {} ms", r.duree_ms);
}

#[test]
fn lancer_commande_introuvable_ou_vide() {
    let e = lancer(&s(&["dsk-commande-qui-n-existe-pas"])).unwrap_err();
    assert!(
        format!("{e:#}").contains("dsk-commande-qui-n-existe-pas"),
        "le message doit dire quelle commande : {e:#}"
    );
    assert!(lancer(&[]).is_err());
}

#[test]
fn rapport_en_json() {
    let r = Rapport {
        commande: "true".to_string(),
        code: Some(0),
        duree_ms: 2,
        stdout: String::new(),
        stderr: String::new(),
    };
    let v: serde_json::Value = serde_json::to_value(&r).unwrap();
    // Les cinq champs, sous ces noms exactement : un script jq en dépend.
    for champ in ["commande", "code", "duree_ms", "stdout", "stderr"] {
        assert!(v.get(champ).is_some(), "champ {champ} absent de {v}");
    }
    assert_eq!(v["code"], 0);
}
