// Labo 06, solution : des comptes bancaires en structs, enums et Option.
// Lancé par : (cd solution && cargo test)   depuis labs/06-compte-bancaire
//
// Rôle du fichier : les types du domaine (Operation, ErreurBanque, Compte,
// Banque) et leurs méthodes. main.rs joue un scénario ; tests/banque.rs
// vérifie chaque règle et chaque message.
//
// Les montants sont des centimes dans un i64 : jamais de f64 pour de
// l'argent, 0.1 + 0.2 n'y vaut pas 0.3. On ne convertit en euros qu'à
// l'affichage.

use std::collections::HashMap;
use std::fmt;

/// Formate des centimes en euros, à la française : `1250` donne `12,50 €`,
/// `-5` donne `-0,05 €`.
pub fn euros(centimes: i64) -> String {
    // Le signe à part : la division entière de -5 par 100 donne 0, qui n'a
    // pas de signe, et on perdrait le moins. unsigned_abs donne la valeur
    // absolue en u64, sans risque de débordement même pour i64::MIN.
    let signe = if centimes < 0 { "-" } else { "" };
    let absolu = centimes.unsigned_abs();
    // {:02} complète à deux chiffres : 5 centimes s'écrivent 05.
    format!("{signe}{},{:02} €", absolu / 100, absolu % 100)
}

/// Une ligne de l'historique d'un compte. Chaque variante porte exactement
/// les données qui la décrivent : un dépôt n'a pas de destinataire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Depot(i64),
    Retrait(i64),
    VirementEmis { vers: String, montant: i64 },
    VirementRecu { de: String, montant: i64 },
}

impl Operation {
    /// Le texte de la ligne de relevé : `dépôt`, `retrait`,
    /// `virement vers bob`, `virement de alice`.
    pub fn libelle(&self) -> String {
        // match sur &self : les motifs lient des références (vers est un
        // &String), on ne déplace rien hors de l'opération.
        match self {
            Operation::Depot(_) => "dépôt".to_string(),
            Operation::Retrait(_) => "retrait".to_string(),
            Operation::VirementEmis { vers, .. } => format!("virement vers {vers}"),
            Operation::VirementRecu { de, .. } => format!("virement de {de}"),
        }
    }

    /// Le montant vu du compte : positif pour ce qui entre, négatif pour ce
    /// qui sort.
    pub fn montant_signe(&self) -> i64 {
        // Les motifs avec `|` regroupent des variantes qui se traitent pareil.
        // Le compilateur vérifie que `montant` existe, avec le même type, dans
        // chaque variante du groupe.
        match self {
            Operation::Depot(montant) | Operation::VirementRecu { montant, .. } => *montant,
            Operation::Retrait(montant) | Operation::VirementEmis { montant, .. } => -*montant,
        }
    }
}

/// Tout ce qui peut mal se passer. Un enum plutôt qu'une chaîne : l'appelant
/// peut tester la variante avec matches! et lire les données (le solde, le
/// montant demandé) sans analyser un texte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurBanque {
    MontantInvalide(i64),
    SoldeInsuffisant { solde: i64, demande: i64 },
    CompteInconnu(String),
    MemeCompte,
}

// Display pour l'humain, Debug (dérivé) pour le développeur. Les messages
// sont en minuscules, sans point final : ils seront peut-être insérés dans
// une phrase plus longue (« virement refusé : solde insuffisant... »).
impl fmt::Display for ErreurBanque {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErreurBanque::MontantInvalide(montant) => {
                write!(
                    f,
                    "montant invalide : {} (il doit être positif)",
                    euros(*montant)
                )
            }
            ErreurBanque::SoldeInsuffisant { solde, demande } => write!(
                f,
                "solde insuffisant : {} disponibles, {} demandés",
                euros(*solde),
                euros(*demande)
            ),
            ErreurBanque::CompteInconnu(titulaire) => write!(f, "compte inconnu : {titulaire}"),
            ErreurBanque::MemeCompte => write!(f, "virement vers le même compte"),
        }
    }
}

/// Un compte. Les champs sont privés : la seule façon de changer le solde est
/// de passer par deposer et retirer, qui vérifient les montants. C'est ce qui
/// garantit l'invariant « le solde n'est jamais négatif ».
#[derive(Debug, Clone)]
pub struct Compte {
    titulaire: String,
    solde_centimes: i64,
    historique: Vec<Operation>,
}

impl Compte {
    /// Un compte vide. Fonction associée (pas de self) : on l'appelle avec
    /// Compte::new("alice").
    pub fn new(titulaire: &str) -> Self {
        Compte {
            titulaire: titulaire.to_string(),
            solde_centimes: 0,
            historique: Vec::new(),
        }
    }

    /// &self : lire ne demande qu'un emprunt partagé. On rend une &str, pas
    /// une &String (chapitre 5 : la forme la plus générale).
    pub fn titulaire(&self) -> &str {
        &self.titulaire
    }

    /// Le solde en centimes. i64 est Copy : on rend une copie, pas une
    /// référence.
    pub fn solde(&self) -> i64 {
        self.solde_centimes
    }

    /// L'historique en lecture seule : une tranche empruntée. L'appelant peut
    /// le parcourir, pas y ajouter une fausse opération.
    pub fn historique(&self) -> &[Operation] {
        &self.historique
    }

    /// Ajoute `montant` centimes. &mut self : on modifie le compte, et
    /// l'appelant doit l'avoir déclaré mut.
    pub fn deposer(&mut self, montant: i64) -> Result<(), ErreurBanque> {
        // Le cas d'erreur d'abord, avec un retour anticipé : la suite de la
        // fonction est le chemin normal, sans indentation.
        if montant <= 0 {
            return Err(ErreurBanque::MontantInvalide(montant));
        }
        self.solde_centimes += montant;
        self.historique.push(Operation::Depot(montant));
        Ok(())
    }

    /// Retire `montant` centimes, si le solde le permet.
    pub fn retirer(&mut self, montant: i64) -> Result<(), ErreurBanque> {
        // On vérifie TOUT avant de modifier quoi que ce soit : en cas
        // d'erreur, le compte est exactement dans l'état d'avant.
        self.verifier_retrait(montant)?;
        self.solde_centimes -= montant;
        self.historique.push(Operation::Retrait(montant));
        Ok(())
    }

    // Les deux contrôles d'un retrait, partagés avec virer. Privée (pas de
    // pub) : c'est un détail d'implémentation. &self : on ne fait que lire.
    fn verifier_retrait(&self, montant: i64) -> Result<(), ErreurBanque> {
        if montant <= 0 {
            return Err(ErreurBanque::MontantInvalide(montant));
        }
        if montant > self.solde_centimes {
            return Err(ErreurBanque::SoldeInsuffisant {
                solde: self.solde_centimes,
                demande: montant,
            });
        }
        Ok(())
    }
}

/// La banque : des comptes rangés par titulaire.
#[derive(Debug, Default)]
pub struct Banque {
    comptes: HashMap<String, Compte>,
}

impl Banque {
    /// Une banque vide. derive(Default) a déjà écrit Banque::default() ; new
    /// est l'alias que tout le monde cherche en premier.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ouvre le compte de `titulaire` s'il n'existe pas, et rend dans tous les
    /// cas une référence mutable vers lui : `banque.ouvrir("alice").deposer(100)`.
    /// Un compte existant n'est pas remis à zéro.
    pub fn ouvrir(&mut self, titulaire: &str) -> &mut Compte {
        // entry fait la recherche une fois ; or_insert_with n'appelle la
        // fermeture (et n'alloue le compte) que si la clé est absente. La
        // référence rendue emprunte self : tant que l'appelant la garde, il
        // ne peut rien faire d'autre avec la banque.
        self.comptes
            .entry(titulaire.to_string())
            .or_insert_with(|| Compte::new(titulaire))
    }

    /// Le compte de `titulaire`, s'il existe. Option<&Compte> : l'absence est
    /// un cas normal, pas une erreur.
    pub fn compte(&self, titulaire: &str) -> Option<&Compte> {
        // HashMap<String, _>::get accepte une &str : String emprunte comme str
        // (le trait Borrow), donc pas besoin de construire une String pour
        // chercher.
        self.comptes.get(titulaire)
    }

    /// Vire `montant` centimes de `de` vers `vers`. Tout ou rien : en cas
    /// d'erreur, aucun des deux comptes n'a changé.
    pub fn virer(&mut self, de: &str, vers: &str, montant: i64) -> Result<(), ErreurBanque> {
        // L'ordre des contrôles est celui des tests : montant, même compte,
        // comptes connus, solde.
        if montant <= 0 {
            return Err(ErreurBanque::MontantInvalide(montant));
        }
        if de == vers {
            return Err(ErreurBanque::MemeCompte);
        }
        // Le cœur du labo. L'idée naïve est :
        //     let source = self.comptes.get_mut(de)?;
        //     let cible = self.comptes.get_mut(vers)?;
        // et le compilateur refuse : error[E0499]: cannot borrow
        // `self.comptes` as mutable more than once at a time. Il ne sait pas
        // que `de` et `vers` sont deux clés différentes ; pour lui, ce sont
        // deux &mut sur la même map, et si les clés étaient égales, on aurait
        // deux références mutables vers le même compte.
        //
        // get_disjoint_mut (Rust 1.86) est fait pour ça : il prend un tableau
        // de clés et rend un tableau d'Option<&mut Compte>. Il vérifie lui-même
        // que les clés sont distinctes (il panique sinon, d'où le test
        // MemeCompte juste au-dessus, qui rend ce cas impossible).
        let [source, cible] = self.comptes.get_disjoint_mut([de, vers]);
        // ok_or transforme le None en erreur, et ? la renvoie tout de suite.
        let source = source.ok_or_else(|| ErreurBanque::CompteInconnu(de.to_string()))?;
        let cible = cible.ok_or_else(|| ErreurBanque::CompteInconnu(vers.to_string()))?;
        // Les deux comptes existent : on contrôle le solde AVANT de toucher à
        // quoi que ce soit.
        source.verifier_retrait(montant)?;

        // Plus rien ne peut échouer : on modifie les deux comptes. Les champs
        // sont privés, mais on est dans le même module que Compte, donc on y a
        // accès.
        source.solde_centimes -= montant;
        source.historique.push(Operation::VirementEmis {
            vers: vers.to_string(),
            montant,
        });
        cible.solde_centimes += montant;
        cible.historique.push(Operation::VirementRecu {
            de: de.to_string(),
            montant,
        });
        Ok(())
        // L'autre solution, sans get_disjoint_mut : vérifier avec des emprunts
        // partagés (contains_key, get), puis faire deux get_mut l'un APRÈS
        // l'autre, chacun dans son propre bloc, pour que le premier &mut soit
        // rendu avant de prendre le second. C'est ce qu'on écrivait avant 1.86.
    }

    /// Le relevé du compte, au format exact :
    ///
    /// ```text
    /// Relevé de alice
    ///   dépôt                      +100,00 €
    ///   virement vers bob           -30,00 €
    /// Solde : 70,00 €
    /// ```
    ///
    /// Chaque ligne d'opération : deux espaces, le libellé aligné à gauche
    /// sur 24 colonnes, le montant signé aligné à droite sur 12. `None` si le
    /// compte n'existe pas.
    pub fn releve(&self, titulaire: &str) -> Option<String> {
        // ? sur Option : compte inconnu, on rend None tout de suite.
        let compte = self.compte(titulaire)?;
        let mut texte = format!("Relevé de {}\n", compte.titulaire());
        for operation in compte.historique() {
            let montant = operation.montant_signe();
            // euros() met un « - » devant les négatifs ; on ajoute le « + »
            // pour les positifs, un relevé l'affiche toujours.
            let signe = if montant >= 0 { "+" } else { "" };
            let montant = format!("{signe}{}", euros(montant));
            // {:<24} et {:>12} comptent en caractères : « € » compte pour un,
            // même s'il prend trois octets.
            texte.push_str(&format!("  {:<24}{:>12}\n", operation.libelle(), montant));
        }
        texte.push_str(&format!("Solde : {}\n", euros(compte.solde())));
        Some(texte)
    }
}
