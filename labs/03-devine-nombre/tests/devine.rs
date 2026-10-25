// Tests du labo 03. Pas de hasard ni de clavier ici : le secret et les
// essais sont passés en paramètres, donc chaque test rend toujours le même
// résultat. Lancés par : cargo test   (un seul groupe : cargo test lire_essai)

use devine::{Resultat, Verdict, evaluer, lire_essai, message, partie};

#[test]
fn evaluer_trois_cas() {
    assert_eq!(evaluer(50, 25), Verdict::TropPetit);
    assert_eq!(evaluer(50, 75), Verdict::TropGrand);
    assert_eq!(evaluer(50, 50), Verdict::Trouve);
}

#[test]
fn evaluer_bornes() {
    assert_eq!(evaluer(1, 1), Verdict::Trouve);
    assert_eq!(evaluer(100, 99), Verdict::TropPetit);
    assert_eq!(evaluer(1, 100), Verdict::TropGrand);
}

#[test]
fn lire_essai_valide() {
    assert_eq!(lire_essai("42"), Some(42));
    assert_eq!(lire_essai("1"), Some(1));
    assert_eq!(lire_essai("100"), Some(100));
}

#[test]
fn lire_essai_blancs() {
    assert_eq!(
        lire_essai("42\n"),
        Some(42),
        "le retour à la ligne de read_line"
    );
    assert_eq!(lire_essai("  7  "), Some(7));
    assert_eq!(lire_essai("\t13\r\n"), Some(13), "fin de ligne Windows");
}

#[test]
fn lire_essai_invalide() {
    assert_eq!(lire_essai(""), None);
    assert_eq!(lire_essai("abc"), None);
    assert_eq!(lire_essai("4 2"), None);
    assert_eq!(lire_essai("-5"), None, "pas de négatif dans un u32");
    assert_eq!(lire_essai("12.5"), None);
}

#[test]
fn lire_essai_hors_bornes() {
    assert_eq!(lire_essai("0"), None);
    assert_eq!(lire_essai("101"), None);
    assert_eq!(lire_essai("99999999999"), None, "plus grand qu'un u32");
}

#[test]
fn partie_gagnee() {
    let r = partie(37, &["50", "25", "37"]);
    assert_eq!(
        r,
        Resultat {
            trouve: true,
            tentatives: 3,
            invalides: 0
        }
    );
}

#[test]
fn partie_premier_coup() {
    let r = partie(64, &["64"]);
    assert!(r.trouve);
    assert_eq!(r.tentatives, 1);
}

#[test]
fn partie_invalides_ne_comptent_pas() {
    let r = partie(10, &["coucou", "50", "", "200", "10"]);
    assert_eq!(
        r,
        Resultat {
            trouve: true,
            tentatives: 2,
            invalides: 3
        }
    );
}

#[test]
fn partie_s_arrete_au_premier_gagnant() {
    let r = partie(5, &["5", "6", "abc", "5"]);
    assert_eq!(
        r,
        Resultat {
            trouve: true,
            tentatives: 1,
            invalides: 0
        }
    );
}

#[test]
fn partie_perdue() {
    let r = partie(99, &["1", "2", "3"]);
    assert_eq!(
        r,
        Resultat {
            trouve: false,
            tentatives: 3,
            invalides: 0
        }
    );
    let vide = partie(99, &[]);
    assert_eq!(
        vide,
        Resultat {
            trouve: false,
            tentatives: 0,
            invalides: 0
        }
    );
}

#[test]
fn partie_accepte_un_vec() {
    // Une &[&str] accepte aussi un Vec<&str> emprunté.
    let lignes: Vec<&str> = "50\n75\n62\n".lines().collect();
    let r = partie(62, &lignes);
    assert!(r.trouve);
    assert_eq!(r.tentatives, 3);
}

#[test]
fn message_indices() {
    assert_eq!(message(Verdict::TropPetit, 1), "C'est plus grand.");
    assert_eq!(message(Verdict::TropGrand, 4), "C'est plus petit.");
}

#[test]
fn message_victoire() {
    assert_eq!(message(Verdict::Trouve, 1), "Trouvé du premier coup !");
    assert_eq!(message(Verdict::Trouve, 2), "Trouvé en 2 tentatives.");
    assert_eq!(message(Verdict::Trouve, 7), "Trouvé en 7 tentatives.");
}
