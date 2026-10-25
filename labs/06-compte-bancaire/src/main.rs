// Labo 06 : un scénario joué sur la petite banque.
// Lancé par : cargo run   (panique sur un todo! tant que lib.rs n'est pas complet)
//
// main ne contient aucune règle : il ouvre des comptes, tente des opérations
// (certaines exprès refusées) et affiche. Tout ce qui décide est dans lib.rs.

use banque::{Banque, euros};

fn main() {
    let mut banque = Banque::new();

    // ouvrir rend un &mut Compte : on enchaîne directement le dépôt. Le
    // résultat du dépôt est un Result ; expect est acceptable ici, un
    // montant positif écrit en dur ne peut pas être refusé (si c'était le
    // cas, ce serait un bug, et le panic le dirait tout de suite).
    banque
        .ouvrir("alice")
        .deposer(10_000)
        .expect("dépôt initial");
    banque.ouvrir("bob").deposer(2_000).expect("dépôt initial");
    println!("alice et bob ouvrent leur compte");

    // Les opérations du scénario : (de, vers, montant). Un tableau de
    // tuples parcouru par référence ; chaque virement rend un Result qu'on
    // traite avec un match, sans rien déballer à l'aveugle.
    let virements = [
        ("alice", "bob", 3_000),
        ("bob", "alice", 9_000),
        ("alice", "carol", 500),
        ("alice", "alice", 100),
        ("bob", "alice", -10),
    ];
    for &(de, vers, montant) in &virements {
        // Display de ErreurBanque donne le message lisible avec {e}.
        match banque.virer(de, vers, montant) {
            Ok(()) => println!("virement {de} -> {vers} de {} : ok", euros(montant)),
            Err(e) => println!("virement {de} -> {vers} refusé : {e}"),
        }
    }

    // Un retrait refusé, puis un retrait accepté. if let Err : seul le cas
    // d'erreur nous intéresse pour l'affichage.
    if let Err(e) = banque.ouvrir("bob").retirer(100_000) {
        println!("retrait bob refusé : {e}");
    }
    banque
        .ouvrir("alice")
        .retirer(1_250)
        .expect("solde suffisant");

    // releve rend un Option<String> : on affiche ceux qui existent. carol
    // n'a jamais été ouverte, le virement refusé ne l'a pas créée.
    println!();
    for titulaire in ["alice", "bob", "carol"] {
        match banque.releve(titulaire) {
            Some(texte) => print!("{texte}"),
            None => println!("pas de compte pour {titulaire}"),
        }
    }
}
