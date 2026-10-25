// Labo 14 : tests du dépôt. Le même scénario est joué sur chaque moteur :
// SQLite en mémoire toujours, SQLite dans un fichier, et PostgreSQL seulement
// si la variable INVENTAIRE_DSN donne un DSN postgres://.
// Lancé par : cargo test
//         ou : INVENTAIRE_DSN=postgres://postgres@127.0.0.1:5432/inventaire cargo test

use inventaire_db::{Article, Depot, DepotPostgres, DepotSqlite, ouvrir};

fn article(nom: &str, quantite: i64) -> Article {
    Article {
        nom: nom.to_string(),
        quantite,
    }
}

/// Le scénario commun. Générique sur le dépôt : le compilateur en fait une
/// version par moteur, et le test ne sait pas lequel il a sous la main.
fn scenario(depot: &mut impl Depot) {
    // Une base neuve est vide.
    assert_eq!(depot.lister().expect("lister"), []);
    assert_eq!(depot.stock("vis").expect("stock"), None);

    // ajouter crée, puis cumule.
    depot.ajouter("vis", 100).expect("ajouter vis");
    depot.ajouter("ecrous", 40).expect("ajouter ecrous");
    depot.ajouter("vis", 20).expect("ajouter encore des vis");
    assert_eq!(depot.stock("vis").expect("stock"), Some(120));

    // lister est trié par nom. Des noms ASCII exprès : l'ordre d'un « é »
    // dépend de la collation de la base (octets pour SQLite, locale du
    // serveur pour PostgreSQL), et un test ne doit pas en dépendre.
    assert_eq!(
        depot.lister().expect("lister"),
        [article("ecrous", 40), article("vis", 120)]
    );

    // retirer rend le reste.
    assert_eq!(depot.retirer("vis", 30).expect("retirer"), 90);
    assert_eq!(depot.retirer("vis", 90).expect("retirer tout"), 0);
    assert_eq!(depot.stock("vis").expect("stock"), Some(0));

    // Stock insuffisant : erreur claire, et le stock n'a pas bougé.
    let erreur = depot.retirer("ecrous", 41).expect_err("41 > 40");
    let message = erreur.to_string();
    assert!(message.contains("stock insuffisant"), "message : {message}");
    assert!(message.contains("40 en stock"), "message : {message}");
    assert_eq!(depot.stock("ecrous").expect("stock"), Some(40));

    // Article inconnu.
    let erreur = depot.retirer("boulons", 1).expect_err("inconnu");
    assert!(
        erreur.to_string().contains("article inconnu : boulons"),
        "message : {erreur}"
    );
    assert_eq!(depot.stock("boulons").expect("stock"), None);

    // Quantités nulles ou négatives refusées des deux côtés.
    for q in [0, -5] {
        let erreur = depot.ajouter("vis", q).expect_err("quantité invalide");
        assert!(erreur.to_string().contains("quantité invalide"), "{erreur}");
        let erreur = depot.retirer("vis", q).expect_err("quantité invalide");
        assert!(erreur.to_string().contains("quantité invalide"), "{erreur}");
    }

    // Les paramètres liés : ce nom reste un nom, la table survit.
    let piege = "x'); DROP TABLE articles; --";
    depot.ajouter(piege, 1).expect("ajouter le nom piégé");
    assert_eq!(depot.stock(piege).expect("stock"), Some(1));
    assert_eq!(depot.lister().expect("la table existe encore").len(), 3);
}

#[test]
fn scenario_sqlite_en_memoire() {
    let mut depot = DepotSqlite::en_memoire().expect("SQLite en mémoire");
    scenario(&mut depot);
}

#[test]
fn sqlite_fichier_persiste() {
    // Un fichier par processus de test, dans le dossier temporaire du système.
    let chemin = std::env::temp_dir().join(format!("lab14-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&chemin);
    {
        let mut depot = DepotSqlite::nouveau(&chemin).expect("ouverture");
        scenario(&mut depot);
    } // la connexion est fermée ici, à la fin du bloc (Drop)
    // Rouvrir : les données sont sur le disque, et la migration rejouée
    // (CREATE TABLE IF NOT EXISTS) ne les efface pas.
    let depot = DepotSqlite::nouveau(&chemin).expect("réouverture");
    assert_eq!(depot.stock("ecrous").expect("stock"), Some(40));
    drop(depot);
    std::fs::remove_file(&chemin).expect("nettoyage");
}

#[test]
fn ouvrir_choisit_le_moteur() {
    let mut depot = ouvrir("sqlite::memory:").expect("sqlite::memory:");
    depot.ajouter("vis", 3).expect("ajouter");
    assert_eq!(depot.stock("vis").expect("stock"), Some(3));

    let erreur = ouvrir("mysql://root:secret@db/inventaire")
        .err()
        .expect("mysql n'est pas géré");
    let message = erreur.to_string();
    assert!(message.contains("DSN non reconnu"), "message : {message}");
    assert!(
        !message.contains("secret"),
        "le mot de passe ne doit pas apparaître : {message}"
    );
}

#[test]
fn scenario_postgres_si_disponible() {
    let dsn = match std::env::var("INVENTAIRE_DSN") {
        Ok(dsn) if dsn.starts_with("postgres") => dsn,
        _ => {
            // Pas de serveur fourni : le test passe en le disant. On voit ce
            // message avec `cargo test -- --nocapture`.
            println!("PostgreSQL ignoré : INVENTAIRE_DSN absente");
            return;
        }
    };
    let mut depot = DepotPostgres::connecter(&dsn).expect("connexion PostgreSQL");
    depot.vider().expect("vider la table");
    scenario(&mut depot);
    depot.vider().expect("nettoyage");
}
