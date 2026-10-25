// Labo 07 : les tests des formes. Identiques dans le squelette et dans la
// solution : `use formes::...` désigne la bibliothèque du paquet.
// Lancé par : cargo test (ton code) ou (cd solution && cargo test).

use formes::*;

// Deux flottants issus de calculs ne sont presque jamais égaux au dernier
// bit près : on compare à un millionième près.
fn proche(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

// Un jeu de formes mélangées, dans le désordre des aires : 12, 3.14..., 6.
fn melange() -> Vec<Box<dyn Forme>> {
    vec![
        Box::new(Rectangle {
            largeur: 3.0,
            hauteur: 4.0,
        }),
        Box::new(Cercle { rayon: 1.0 }),
        Box::new(Triangle::new(3.0, 4.0, 5.0).expect("triangle 3-4-5")),
    ]
}

#[test]
fn cercle_aire_et_perimetre() {
    let c = Cercle { rayon: 2.0 };
    assert!(proche(c.aire(), 12.566370614359172), "aire = {}", c.aire());
    assert!(proche(c.perimetre(), 12.566370614359172));
    assert_eq!(c.nom(), "cercle");
}

#[test]
fn rectangle_aire_perimetre_et_from() {
    let r = Rectangle::from((3.0, 4.0));
    assert_eq!(
        r,
        Rectangle {
            largeur: 3.0,
            hauteur: 4.0
        }
    );
    assert!(proche(r.aire(), 12.0));
    assert!(proche(r.perimetre(), 14.0));
    // From donne Into : le type cible est connu grâce à l'annotation.
    let r2: Rectangle = (2.5, 2.0).into();
    assert!(proche(r2.aire(), 5.0));
    assert_eq!(r2.nom(), "rectangle");
}

#[test]
fn triangle_heron() {
    let t = Triangle::new(3.0, 4.0, 5.0).expect("triangle valide");
    assert!(proche(t.aire(), 6.0), "aire = {}", t.aire());
    assert!(proche(t.perimetre(), 12.0));
    let equilateral = Triangle::new(2.0, 2.0, 2.0).expect("triangle valide");
    assert!(proche(equilateral.aire(), 3f64.sqrt()));
    // Plat (1 + 2 = 3) : accepté, aire nulle.
    let plat = Triangle::new(1.0, 2.0, 3.0).expect("triangle plat");
    assert!(proche(plat.aire(), 0.0));
    assert_eq!(t.nom(), "triangle");
}

#[test]
fn triangle_invalide() {
    assert_eq!(
        Triangle::new(1.0, 2.0, 10.0),
        None,
        "inégalité triangulaire"
    );
    assert_eq!(Triangle::new(0.0, 1.0, 1.0), None, "côté nul");
    assert_eq!(Triangle::new(-3.0, 4.0, 5.0), None, "côté négatif");
    assert_eq!(Triangle::new(f64::NAN, 1.0, 1.0), None, "NaN");
}

#[test]
fn affichage() {
    assert_eq!(Cercle { rayon: 2.0 }.to_string(), "cercle de rayon 2");
    assert_eq!(Cercle { rayon: 0.5 }.to_string(), "cercle de rayon 0.5");
    assert_eq!(Rectangle::from((3.0, 4.5)).to_string(), "rectangle 3 x 4.5");
    assert_eq!(
        Triangle::new(3.0, 4.0, 5.0).unwrap().to_string(),
        "triangle 3-4-5"
    );
}

#[test]
fn decrire_par_defaut() {
    assert_eq!(
        Rectangle::from((3.0, 4.0)).decrire(),
        "rectangle : aire 12.00, périmètre 14.00"
    );
    assert_eq!(
        Cercle { rayon: 1.0 }.decrire(),
        "cercle : aire 3.14, périmètre 6.28"
    );
}

#[test]
fn plus_grande_forme() {
    let formes = melange();
    let grande = plus_grande(&formes).expect("liste non vide");
    assert_eq!(grande.nom(), "rectangle");
    assert!(proche(grande.aire(), 12.0));
    assert!(plus_grande(&[]).is_none());
}

#[test]
fn aire_totale_generique_et_dyn() {
    let rectangles = [Rectangle::from((1.0, 2.0)), Rectangle::from((3.0, 4.0))];
    assert!(proche(aire_totale(&rectangles), 14.0));
    let cercles = [Cercle { rayon: 1.0 }];
    assert!(proche(aire_totale(&cercles), std::f64::consts::PI));
    let vide: [Cercle; 0] = [];
    assert!(proche(aire_totale(&vide), 0.0));

    let formes = melange();
    assert!(proche(
        aire_totale_dyn(&formes),
        12.0 + std::f64::consts::PI + 6.0
    ));
    assert!(proche(aire_totale_dyn(&[]), 0.0));
}

#[test]
fn trier_par_aire_croissante() {
    let mut formes = melange();
    trier_par_aire(&mut formes);
    let noms: Vec<String> = formes.iter().map(|f| f.nom()).collect();
    assert_eq!(noms, ["cercle", "triangle", "rectangle"]);
}

#[test]
fn rapport_dans_un_tampon() {
    let formes = melange();
    let mut tampon = Tampon::default();
    rapport(&formes, &mut tampon);
    assert_eq!(
        tampon.0,
        [
            "1. rectangle : aire 12.00, périmètre 14.00",
            "2. cercle : aire 3.14, périmètre 6.28",
            "3. triangle : aire 6.00, périmètre 12.00",
            "total : 3 formes, aire 21.14",
            "plus grande : rectangle",
        ]
    );
}

#[test]
fn rapport_vide() {
    let mut tampon = Tampon(Vec::new());
    rapport(&[], &mut tampon);
    assert_eq!(tampon.0, ["aucune forme"]);
}

// Ne vérifie que la compilation : Console doit implémenter Sortie.
#[test]
fn console_est_une_sortie() {
    fn accepte(_sortie: &mut impl Sortie) {}
    accepte(&mut Console);
}

#[test]
fn compteur_est_un_iterateur() {
    let tout: Vec<u32> = Compteur::new(1, 5).collect();
    assert_eq!(tout, [1, 2, 3, 4]);
    assert_eq!(Compteur { courant: 3, fin: 3 }.count(), 0);
    assert_eq!(Compteur::new(9, 2).next(), None, "début après la fin");
    // Les méthodes par défaut d'Iterator viennent gratuitement.
    let somme_carres: u32 = Compteur::new(1, 11).map(|n| n * n).sum();
    assert_eq!(somme_carres, 385);
    let pairs: Vec<u32> = Compteur::new(0, 10).filter(|n| n % 2 == 0).collect();
    assert_eq!(pairs, [0, 2, 4, 6, 8]);
}

#[test]
fn compteur_reste_fini() {
    let mut c = Compteur::new(0, 1);
    assert_eq!(c.next(), Some(0));
    assert_eq!(c.next(), None);
    assert_eq!(c.next(), None, "un itérateur fini le reste");
}
