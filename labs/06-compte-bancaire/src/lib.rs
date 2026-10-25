// Labo 06 : des comptes bancaires en structs, enums et Option.
// Rôle du fichier : les types du domaine (Operation, ErreurBanque, Compte,
// Banque) et leurs méthodes. main.rs joue un scénario ; tests/banque.rs
// vérifie chaque règle et chaque message. Les types sont écrits ; à toi les
// corps de fonctions, dans l'ordre des TODO.
// Lancé par : cargo test   (tout est rouge au départ, mais tout compile)
//
// Les montants sont des centimes dans un i64 : jamais de f64 pour de
// l'argent. On ne convertit en euros qu'à l'affichage.

use std::collections::HashMap;
use std::fmt;

/// Formate des centimes en euros, à la française : `1250` donne `12,50 €`,
/// `-5` donne `-0,05 €`.
pub fn euros(centimes: i64) -> String {
    // TODO 1 : sépare le signe (« - » si centimes < 0, sinon rien) et la
    // valeur absolue (centimes.unsigned_abs(), un u64). Les euros sont
    // absolu / 100, les centimes absolu % 100, écrits sur deux chiffres avec
    // {:02}. Piège : -5 / 100 vaut 0, et 0 n'a pas de signe, d'où le signe à
    // part. format!("{signe}{},{:02} €", ...).
    let _ = centimes;
    todo!("TODO 1")
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
        // TODO 8 : un match self avec un bras par variante.
        // `Operation::VirementEmis { vers, .. } => format!("virement vers {vers}")`.
        todo!("TODO 8")
    }

    /// Le montant vu du compte : positif pour ce qui entre (dépôt, virement
    /// reçu), négatif pour ce qui sort (retrait, virement émis).
    pub fn montant_signe(&self) -> i64 {
        // TODO 8 (suite) : un match aussi. Un motif peut regrouper deux
        // variantes avec `|` si elles lient le même nom :
        // `Operation::Depot(montant) | Operation::VirementRecu { montant, .. } => *montant`.
        todo!("TODO 8")
    }
}

/// Tout ce qui peut mal se passer. Un enum plutôt qu'une chaîne : l'appelant
/// teste la variante avec matches! et lit les données sans analyser un texte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurBanque {
    MontantInvalide(i64),
    SoldeInsuffisant { solde: i64, demande: i64 },
    CompteInconnu(String),
    MemeCompte,
}

impl fmt::Display for ErreurBanque {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO 2 : un match self, un write!(f, ...) par variante. Les quatre
        // messages exacts sont dans le test messages_d_erreur, par exemple
        // « solde insuffisant : 10,00 € disponibles, 25,00 € demandés ».
        // Les montants passent par euros(*solde) : dans le match sur &self,
        // solde est un &i64, l'étoile le copie.
        let _ = f;
        todo!("TODO 2")
    }
}

/// Un compte. Les champs sont privés : la seule façon de changer le solde est
/// de passer par deposer et retirer, qui vérifient les montants.
#[derive(Debug, Clone)]
pub struct Compte {
    titulaire: String,
    solde_centimes: i64,
    historique: Vec<Operation>,
}

impl Compte {
    /// Un compte vide.
    pub fn new(titulaire: &str) -> Self {
        Compte {
            titulaire: titulaire.to_string(),
            solde_centimes: 0,
            historique: Vec::new(),
        }
    }

    pub fn titulaire(&self) -> &str {
        &self.titulaire
    }

    /// Le solde en centimes.
    pub fn solde(&self) -> i64 {
        self.solde_centimes
    }

    /// L'historique en lecture seule : une tranche empruntée.
    pub fn historique(&self) -> &[Operation] {
        &self.historique
    }

    /// Ajoute `montant` centimes ; refuse un montant nul ou négatif.
    pub fn deposer(&mut self, montant: i64) -> Result<(), ErreurBanque> {
        // TODO 3 : si montant <= 0, return Err(ErreurBanque::MontantInvalide(montant)).
        // Sinon, ajoute au solde, pousse Operation::Depot(montant) dans
        // l'historique, et rends Ok(()). Un dépôt refusé ne laisse aucune trace.
        let _ = montant;
        todo!("TODO 3")
    }

    /// Retire `montant` centimes, si le solde le permet.
    pub fn retirer(&mut self, montant: i64) -> Result<(), ErreurBanque> {
        // TODO 4 : vérifie TOUT avant de modifier : montant <= 0 donne
        // MontantInvalide, montant > solde donne SoldeInsuffisant { solde, demande }.
        // Ces deux contrôles serviront aussi à virer (TODO 7) : écris-les dans
        // une méthode privée `fn verifier_retrait(&self, montant: i64) -> Result<(), ErreurBanque>`
        // et appelle-la ici avec `self.verifier_retrait(montant)?;`.
        let _ = montant;
        todo!("TODO 4")
    }
}

/// La banque : des comptes rangés par titulaire.
#[derive(Debug, Default)]
pub struct Banque {
    comptes: HashMap<String, Compte>,
}

impl Banque {
    /// Une banque vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ouvre le compte de `titulaire` s'il n'existe pas, et rend dans tous les
    /// cas une référence mutable vers lui. Un compte existant n'est pas remis
    /// à zéro.
    pub fn ouvrir(&mut self, titulaire: &str) -> &mut Compte {
        // TODO 5 : l'API entry du chapitre 5 :
        // self.comptes.entry(titulaire.to_string()).or_insert_with(|| Compte::new(titulaire)).
        // or_insert_with rend justement le &mut Compte qu'on veut renvoyer.
        let _ = (&self.comptes, titulaire);
        todo!("TODO 5")
    }

    /// Le compte de `titulaire`, s'il existe.
    pub fn compte(&self, titulaire: &str) -> Option<&Compte> {
        // TODO 6 : HashMap::get. Une HashMap<String, _> se consulte avec une
        // &str, pas besoin de fabriquer une String.
        let _ = titulaire;
        todo!("TODO 6")
    }

    /// Vire `montant` centimes de `de` vers `vers`. Tout ou rien : en cas
    /// d'erreur, aucun des deux comptes n'a changé.
    pub fn virer(&mut self, de: &str, vers: &str, montant: i64) -> Result<(), ErreurBanque> {
        // TODO 7 : le cœur du labo. Les contrôles, dans cet ordre (c'est celui
        // des tests) : montant <= 0 (MontantInvalide), de == vers (MemeCompte),
        // comptes inconnus (CompteInconnu, la source d'abord), solde (avec
        // verifier_retrait). Puis on modifie les deux comptes : solde et
        // historique (VirementEmis chez la source, VirementRecu chez la cible).
        //
        // Essaie d'abord l'idée naïve, deux self.comptes.get_mut(...) gardés
        // en même temps, et lis l'erreur E0499 : deux &mut sur la même map.
        // Deux sorties propres :
        //  - self.comptes.get_disjoint_mut([de, vers]) (Rust 1.86) rend
        //    [Option<&mut Compte>; 2] et vérifie que les clés diffèrent (il
        //    panique sinon : le test MemeCompte d'avant l'évite) ;
        //  - ou tout vérifier avec des emprunts partagés (get, contains_key),
        //    puis deux get_mut successifs, chacun dans son bloc.
        // Option -> Result : .ok_or_else(|| ErreurBanque::CompteInconnu(de.to_string()))?.
        let _ = (de, vers, montant);
        todo!("TODO 7")
    }

    /// Le relevé du compte, au format exact (voir le test releve_au_format_exact) :
    ///
    /// ```text
    /// Relevé de alice
    ///   dépôt                      +100,00 €
    ///   virement vers bob           -30,00 €
    /// Solde : 70,00 €
    /// ```
    ///
    /// Chaque ligne d'opération : deux espaces, le libellé aligné à gauche
    /// sur 24 colonnes, le montant signé aligné à droite sur 12, un « + »
    /// devant les montants positifs. `None` si le compte n'existe pas.
    pub fn releve(&self, titulaire: &str) -> Option<String> {
        // TODO 9 : let compte = self.compte(titulaire)?; (le ? sur Option).
        // Puis une String : l'en-tête, une ligne par opération avec
        // format!("  {:<24}{:>12}\n", operation.libelle(), montant) où montant
        // est le texte signé déjà construit, et la ligne « Solde : ... ».
        // Chaque ligne finit par \n, la dernière aussi.
        let _ = titulaire;
        todo!("TODO 9")
    }
}
