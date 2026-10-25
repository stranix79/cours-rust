// Labo 06 : les tests de la banque. Identiques dans le squelette et dans la
// solution : `use banque::...` désigne la bibliothèque du paquet.
// Lancé par : cargo test (ton code) ou (cd solution && cargo test).

use banque::*;

// Une banque avec alice (100,00 €) et bob (20,00 €), pour les tests de virement.
fn banque_de_test() -> Banque {
    let mut b = Banque::new();
    b.ouvrir("alice").deposer(10_000).expect("dépôt alice");
    b.ouvrir("bob").deposer(2_000).expect("dépôt bob");
    b
}

// Le solde d'un compte qui doit exister.
fn solde(b: &Banque, titulaire: &str) -> i64 {
    b.compte(titulaire).expect("le compte doit exister").solde()
}

#[test]
fn euros_formate_a_la_francaise() {
    assert_eq!(euros(0), "0,00 €");
    assert_eq!(euros(5), "0,05 €");
    assert_eq!(euros(1250), "12,50 €");
    assert_eq!(euros(100_000), "1000,00 €");
    assert_eq!(euros(-5), "-0,05 €", "le signe ne doit pas se perdre");
    assert_eq!(euros(-3_000), "-30,00 €");
}

#[test]
fn messages_d_erreur() {
    assert_eq!(
        ErreurBanque::MontantInvalide(-5).to_string(),
        "montant invalide : -0,05 € (il doit être positif)"
    );
    assert_eq!(
        ErreurBanque::MontantInvalide(0).to_string(),
        "montant invalide : 0,00 € (il doit être positif)"
    );
    assert_eq!(
        ErreurBanque::SoldeInsuffisant {
            solde: 1_000,
            demande: 2_500
        }
        .to_string(),
        "solde insuffisant : 10,00 € disponibles, 25,00 € demandés"
    );
    assert_eq!(
        ErreurBanque::CompteInconnu("carol".to_string()).to_string(),
        "compte inconnu : carol"
    );
    assert_eq!(
        ErreurBanque::MemeCompte.to_string(),
        "virement vers le même compte"
    );
}

#[test]
fn deposer_ajoute_et_note_l_operation() {
    let mut c = Compte::new("alice");
    assert_eq!(c.titulaire(), "alice");
    assert_eq!(c.solde(), 0);
    assert!(c.historique().is_empty());
    c.deposer(1_000).expect("dépôt positif");
    c.deposer(250).expect("dépôt positif");
    assert_eq!(c.solde(), 1_250);
    assert_eq!(
        c.historique(),
        [Operation::Depot(1_000), Operation::Depot(250)]
    );
}

#[test]
fn deposer_refuse_zero_et_negatif() {
    let mut c = Compte::new("alice");
    assert_eq!(c.deposer(0), Err(ErreurBanque::MontantInvalide(0)));
    assert!(matches!(
        c.deposer(-300),
        Err(ErreurBanque::MontantInvalide(-300))
    ));
    assert_eq!(c.solde(), 0);
    assert!(
        c.historique().is_empty(),
        "un dépôt refusé ne laisse pas de trace"
    );
}

#[test]
fn retirer_dans_la_limite_du_solde() {
    let mut c = Compte::new("alice");
    c.deposer(1_000).expect("dépôt");
    c.retirer(400).expect("retrait possible");
    c.retirer(600).expect("retrait de tout le reste");
    assert_eq!(c.solde(), 0);
    assert_eq!(
        c.historique(),
        [
            Operation::Depot(1_000),
            Operation::Retrait(400),
            Operation::Retrait(600)
        ]
    );
}

#[test]
fn retirer_refuse_sans_rien_changer() {
    let mut c = Compte::new("alice");
    c.deposer(1_000).expect("dépôt");
    assert_eq!(
        c.retirer(2_500),
        Err(ErreurBanque::SoldeInsuffisant {
            solde: 1_000,
            demande: 2_500
        })
    );
    assert_eq!(c.retirer(-1), Err(ErreurBanque::MontantInvalide(-1)));
    assert_eq!(c.retirer(0), Err(ErreurBanque::MontantInvalide(0)));
    assert_eq!(c.solde(), 1_000, "un retrait refusé ne touche pas au solde");
    assert_eq!(c.historique().len(), 1);
}

#[test]
fn ouvrir_ne_remet_pas_a_zero() {
    let mut b = Banque::new();
    assert!(b.compte("alice").is_none());
    b.ouvrir("alice").deposer(500).expect("dépôt");
    // Rouvrir un compte existant rend le même compte, solde intact.
    let c = b.ouvrir("alice");
    assert_eq!(c.solde(), 500);
    assert_eq!(c.titulaire(), "alice");
    assert_eq!(solde(&b, "alice"), 500);
    assert!(b.compte("bob").is_none());
}

#[test]
fn virer_deplace_l_argent_et_l_historique() {
    let mut b = banque_de_test();
    b.virer("alice", "bob", 3_000).expect("virement possible");
    assert_eq!(solde(&b, "alice"), 7_000);
    assert_eq!(solde(&b, "bob"), 5_000);
    assert_eq!(
        b.compte("alice").unwrap().historique().last(),
        Some(&Operation::VirementEmis {
            vers: "bob".to_string(),
            montant: 3_000
        })
    );
    assert_eq!(
        b.compte("bob").unwrap().historique().last(),
        Some(&Operation::VirementRecu {
            de: "alice".to_string(),
            montant: 3_000
        })
    );
}

#[test]
fn virer_refuse_et_ne_change_rien() {
    let mut b = banque_de_test();
    assert_eq!(
        b.virer("alice", "bob", 0),
        Err(ErreurBanque::MontantInvalide(0))
    );
    assert_eq!(
        b.virer("alice", "alice", 100),
        Err(ErreurBanque::MemeCompte)
    );
    assert_eq!(
        b.virer("carol", "bob", 100),
        Err(ErreurBanque::CompteInconnu("carol".to_string()))
    );
    assert_eq!(
        b.virer("alice", "dave", 100),
        Err(ErreurBanque::CompteInconnu("dave".to_string()))
    );
    let err = b.virer("bob", "alice", 5_000).unwrap_err();
    assert!(
        matches!(
            err,
            ErreurBanque::SoldeInsuffisant {
                solde: 2_000,
                demande: 5_000
            }
        ),
        "obtenu {err:?}"
    );
    // Aucun des refus n'a laissé de trace : tout ou rien.
    assert_eq!(solde(&b, "alice"), 10_000);
    assert_eq!(solde(&b, "bob"), 2_000);
    assert_eq!(b.compte("alice").unwrap().historique().len(), 1);
    assert_eq!(b.compte("bob").unwrap().historique().len(), 1);
    assert!(
        b.compte("dave").is_none(),
        "un virement refusé n'ouvre pas de compte"
    );
}

#[test]
fn releve_au_format_exact() {
    let mut b = banque_de_test();
    b.virer("alice", "bob", 3_000).expect("virement");
    b.ouvrir("alice").retirer(1_250).expect("retrait");
    let attendu = "\
Relevé de alice
  dépôt                      +100,00 €
  virement vers bob           -30,00 €
  retrait                     -12,50 €
Solde : 57,50 €
";
    assert_eq!(b.releve("alice").as_deref(), Some(attendu));

    let attendu_bob = "\
Relevé de bob
  dépôt                       +20,00 €
  virement de alice           +30,00 €
Solde : 50,00 €
";
    assert_eq!(b.releve("bob").as_deref(), Some(attendu_bob));
}

#[test]
fn releve_compte_vide_et_inconnu() {
    let mut b = Banque::new();
    b.ouvrir("carol");
    assert_eq!(
        b.releve("carol").as_deref(),
        Some("Relevé de carol\nSolde : 0,00 €\n")
    );
    assert_eq!(b.releve("personne"), None);
}
