// Tests du labo 04. Chaque test construit son propre inventaire : aucune
// donnée partagée, aucun ordre d'exécution à respecter.
// Lancés par : cargo test   (un seul groupe : cargo test retirer)

use inventaire::{
    Article, ajouter, chercher, format_tableau, fusionner, noms, plus_cher, retirer, valeur_totale,
};

// Fabrique un article : le nom arrive en &str, l'article possède sa String.
fn article(nom: &str, quantite: u32, prix_centimes: u64) -> Article {
    Article {
        nom: nom.to_string(),
        quantite,
        prix_centimes,
    }
}

fn exemple() -> Vec<Article> {
    vec![
        article("clavier", 3, 4990),
        article("souris", 5, 1990),
        article("écran", 1, 18900),
    ]
}

#[test]
fn ajouter_cree_un_article() {
    let mut inv = Vec::new();
    ajouter(&mut inv, "clavier", 2, 4990);
    assert_eq!(inv, vec![article("clavier", 2, 4990)]);
}

#[test]
fn ajouter_fusionne_les_quantites() {
    let mut inv = exemple();
    ajouter(&mut inv, "souris", 3, 9999);
    assert_eq!(inv.len(), 3, "pas de doublon");
    assert_eq!(
        inv[1],
        article("souris", 8, 1990),
        "le prix d'origine reste"
    );
}

#[test]
fn ajouter_garde_l_ordre() {
    let mut inv = exemple();
    ajouter(&mut inv, "câble", 4, 590);
    assert_eq!(noms(&inv), vec!["clavier", "souris", "écran", "câble"]);
}

#[test]
fn ajouter_accepte_une_string_empruntee() {
    let mut inv = Vec::new();
    let nom = String::from("webcam");
    ajouter(&mut inv, &nom, 1, 3900);
    // nom est toujours à nous : ajouter n'a fait que l'emprunter.
    assert_eq!(nom, "webcam");
    assert_eq!(inv[0].nom, "webcam");
}

#[test]
fn chercher_trouve_et_ne_trouve_pas() {
    let inv = exemple();
    assert_eq!(chercher(&inv, "souris"), Some(&article("souris", 5, 1990)));
    assert_eq!(chercher(&inv, "webcam"), None);
    assert_eq!(chercher(&[], "souris"), None);
}

#[test]
fn chercher_survit_au_nom() {
    let inv = exemple();
    let trouve;
    {
        // Le nom cherché meurt à la fin de ce bloc ; le résultat, qui
        // emprunte à inv et pas au nom, reste utilisable après.
        let nom = String::from("écran");
        trouve = chercher(&inv, &nom);
    }
    assert_eq!(trouve.map(|a| a.prix_centimes), Some(18900));
}

#[test]
fn retirer_rend_le_reste() {
    let mut inv = exemple();
    assert_eq!(retirer(&mut inv, "souris", 2), Ok(3));
    assert_eq!(chercher(&inv, "souris").map(|a| a.quantite), Some(3));
}

#[test]
fn retirer_supprime_a_zero() {
    let mut inv = exemple();
    assert_eq!(retirer(&mut inv, "clavier", 3), Ok(0));
    assert_eq!(chercher(&inv, "clavier"), None);
    assert_eq!(noms(&inv), vec!["souris", "écran"]);
}

#[test]
fn retirer_article_inconnu() {
    let mut inv = exemple();
    let erreur = retirer(&mut inv, "webcam", 1).unwrap_err();
    assert!(
        erreur.contains("webcam"),
        "le message cite le nom : {erreur}"
    );
    assert_eq!(inv, exemple(), "rien n'a changé");
}

#[test]
fn retirer_stock_insuffisant() {
    let mut inv = exemple();
    let erreur = retirer(&mut inv, "écran", 2).unwrap_err();
    assert!(
        erreur.contains("écran"),
        "le message cite le nom : {erreur}"
    );
    assert_eq!(inv, exemple(), "rien n'a changé");
}

#[test]
fn valeur_totale_calcule() {
    // 3 × 49.90 + 5 × 19.90 + 1 × 189.00 = 149.70 + 99.50 + 189.00
    assert_eq!(valeur_totale(&exemple()), 43820);
    assert_eq!(valeur_totale(&[]), 0);
}

#[test]
fn valeur_totale_sur_une_tranche() {
    let inv = exemple();
    // Une &[Article] accepte aussi un morceau du Vec.
    assert_eq!(valeur_totale(&inv[1..]), 9950 + 18900);
}

#[test]
fn valeur_totale_sans_debordement_u32() {
    // 100 000 × 100 000 centimes dépasse u32::MAX : le calcul doit être en u64.
    let inv = vec![article("serveur", 100_000, 100_000)];
    assert_eq!(valeur_totale(&inv), 10_000_000_000);
}

#[test]
fn plus_cher_trouve() {
    let inv = exemple();
    assert_eq!(plus_cher(&inv).map(|a| a.nom.as_str()), Some("écran"));
    assert_eq!(plus_cher(&[]), None);
}

#[test]
fn noms_dans_l_ordre() {
    let inv = exemple();
    assert_eq!(noms(&inv), vec!["clavier", "souris", "écran"]);
    assert!(noms(&[]).is_empty());
}

#[test]
fn fusionner_additionne_et_ajoute() {
    let a = exemple();
    let b = vec![article("souris", 10, 1000), article("câble", 4, 590)];
    let tout = fusionner(a, b);
    assert_eq!(
        tout,
        vec![
            article("clavier", 3, 4990),
            article("souris", 15, 1990),
            article("écran", 1, 18900),
            article("câble", 4, 590),
        ]
    );
}

#[test]
fn fusionner_avec_vide() {
    assert_eq!(fusionner(exemple(), Vec::new()), exemple());
    assert_eq!(fusionner(Vec::new(), exemple()), exemple());
}

#[test]
fn format_tableau_exact() {
    let inv = vec![article("clavier", 2, 4990), article("écran", 1, 18900)];
    let attendu = "\
Article       Qté      Prix     Total
clavier         2     49.90     99.80
écran           1    189.00    189.00
TOTAL                          288.80
";
    assert_eq!(format_tableau(&inv), attendu);
}

#[test]
fn format_tableau_vide() {
    let attendu = "\
Article       Qté      Prix     Total
TOTAL                            0.00
";
    assert_eq!(format_tableau(&[]), attendu);
}

#[test]
fn format_tableau_centimes() {
    let inv = vec![article("vis", 7, 5)];
    assert!(
        format_tableau(&inv).contains("vis             7      0.05      0.35\n"),
        "{}",
        format_tableau(&inv)
    );
}
