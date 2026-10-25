// Labo 04 : un inventaire d'articles, pour faire travailler la possession
// et les emprunts.
// Testé par : cargo test   Lancé par : cargo run
//
// Complète les TODO dans l'ordre. Tout compile dès maintenant : chaque
// fonction à écrire contient un todo!(), qui panique à l'exécution.
//
// Les signatures sont données, et c'est le cœur du labo : avant d'écrire un
// corps, lis la signature et le commentaire qui explique chaque `&`, `&mut`
// et `'a`. La règle à garder sous les yeux :
//   - `T`       : la fonction prend possession (l'appelant ne l'a plus) ;
//   - `&T`      : la fonction lit, l'appelant garde la valeur ;
//   - `&mut T`  : la fonction modifie, l'appelant garde la valeur, et
//                 personne d'autre n'y touche pendant l'appel.
// Pour t'entraîner, change une signature (retire un &, ajoute un mut) et
// lis l'erreur du compilateur, puis remets-la.

// Les paramètres des fonctions pas encore écrites sont inutilisés, ce que
// le compilateur signale. Retire cette ligne quand tous les TODO sont faits.
#![allow(unused_variables)]
// Tant que ajouter et retirer ne contiennent qu'un todo!(), clippy ne voit
// aucun push ni remove, et croit qu'une tranche &mut [Article] suffirait.
// Cette ligne aussi est à retirer à la fin : clippy sera alors d'accord.
#![allow(clippy::ptr_arg)]

/// Un article en stock.
// Pas de Copy : l'article contient une String, qui possède un bloc sur le
// tas. Clone permet de le dupliquer quand on le décide, avec .clone().
#[derive(Debug, Clone, PartialEq)]
pub struct Article {
    /// Le nom, possédé par l'article (String, pas &str) : l'article ne
    /// dépend ainsi d'aucun texte extérieur.
    pub nom: String,
    pub quantite: u32,
    /// Le prix unitaire en centimes : de l'argent dans un entier.
    pub prix_centimes: u64,
}

/// Ajoute `quantite` articles. Si un article du même nom existe déjà, sa
/// quantité augmente et son prix ne change pas ; sinon l'article est créé.
// `&mut Vec<Article>` : on modifie le Vec de l'appelant, et on peut devoir y
// pousser un élément, donc il faut le Vec lui-même (une tranche &mut
// [Article] ne peut pas grandir). `&str` pour le nom : on ne fait que le
// lire, et on n'en fabrique une String possédée que pour un nouvel article.
pub fn ajouter(inv: &mut Vec<Article>, nom: &str, quantite: u32, prix_centimes: u64) {
    // TODO 1 : cherche un article du même nom avec
    // `if let Some(article) = inv.iter_mut().find(|a| a.nom == nom)` :
    // iter_mut() donne des &mut Article, que tu peux modifier sur place.
    // Sinon (else), inv.push(Article { nom: nom.to_string(), ... }).
    // Une boucle `for article in inv.iter_mut()` avec un `return` dès que
    // tu as trouvé marche aussi.
    todo!("TODO 1")
}

/// Cherche un article par son nom.
// Deux références en entrée, une en sortie : le compilateur ne peut pas
// deviner à laquelle la sortie emprunte. 'a le dit : le résultat pointe
// dans `inv`, pas dans `nom`. L'appelant peut donc jeter `nom` juste après
// l'appel et garder le résultat (le test chercher_survit_au_nom le vérifie).
// Retire les 'a pour voir l'erreur E0106.
pub fn chercher<'a>(inv: &'a [Article], nom: &str) -> Option<&'a Article> {
    // TODO 2 : inv.iter() donne des &Article ; .find(|a| a.nom == nom)
    // rend le premier qui correspond, ou None. Une ligne. Ou une boucle
    // `for a in inv { if ... { return Some(a); } }` puis None.
    todo!("TODO 2")
}

/// Retire `quantite` articles et rend la quantité restante. Un article qui
/// tombe à zéro disparaît de l'inventaire. Erreur si l'article est inconnu
/// ou si le stock ne suffit pas (et alors rien n'est modifié).
// `&mut Vec<Article>` : on peut supprimer un élément, donc changer la
// longueur du Vec.
pub fn retirer(inv: &mut Vec<Article>, nom: &str, quantite: u32) -> Result<u32, String> {
    // TODO 3 : trouve l'INDICE de l'article, pas une référence :
    // inv.iter().position(|a| a.nom == nom) rend Option<usize>. Avec une
    // &mut Article encore utilisée après inv.remove(i), le compilateur
    // refuserait (E0499 : deux emprunts mutables de inv). Un usize
    // n'emprunte rien.
    // - inconnu : Err(format!("article inconnu : {nom}")) ;
    // - stock insuffisant : un Err qui cite le nom ;
    // - reste 0 : inv.remove(i) ; sinon inv[i].quantite = reste ;
    // - Ok(reste).
    todo!("TODO 3")
}

/// La valeur du stock : somme des quantités multipliées par les prix.
// `&[Article]` : une tranche empruntée en lecture. Elle accepte &Vec,
// un morceau de Vec (&inv[1..]) ou un tableau. &Vec<Article> serait plus
// restrictif sans rien apporter.
pub fn valeur_totale(inv: &[Article]) -> u64 {
    // TODO 4 : `for article in inv` donne des &Article. Calcule en u64 :
    // u64::from(article.quantite) * article.prix_centimes (un u32 × u32
    // déborderait vite, le test valeur_totale_sans_debordement_u32 le
    // vérifie).
    todo!("TODO 4")
}

/// L'article au prix unitaire le plus élevé, ou None si l'inventaire est
/// vide.
// Une seule référence en entrée, une en sortie : les règles d'élision
// s'appliquent, la sortie emprunte à `inv`. Écrite en entier, la signature
// serait fn plus_cher<'a>(inv: &'a [Article]) -> Option<&'a Article>.
pub fn plus_cher(inv: &[Article]) -> Option<&Article> {
    // TODO 5 : inv.iter().max_by_key(|a| a.prix_centimes). Ou une boucle
    // qui garde le meilleur dans un `let mut meilleur: Option<&Article> = None;`.
    todo!("TODO 5")
}

/// Les noms des articles, dans l'ordre de l'inventaire.
// Vec<&str> : des vues sur les String des articles, aucune copie de texte.
// L'élision donne à ces &str la durée de vie de `inv` (même règle que
// plus_cher). Tant que le résultat sert, l'appelant ne peut pas modifier
// l'inventaire : essaie un ajouter() entre noms() et l'affichage, E0502.
pub fn noms(inv: &[Article]) -> Vec<&str> {
    // TODO 6 : un Vec::with_capacity(inv.len()), puis pour chaque article
    // push(article.nom.as_str()).
    todo!("TODO 6")
}

/// Fusionne deux inventaires en un seul : les articles de `b` s'ajoutent à
/// ceux de `a`, les quantités des noms communs s'additionnent (le prix de
/// `a` est gardé). L'ordre est celui de `a`, puis les nouveaux de `b`.
// Pas de & : la fonction PREND les deux Vec. L'appelant ne peut plus s'en
// servir après l'appel (E0382 s'il essaie), et en échange on déplace les
// articles de b dans a sans cloner une seule String.
pub fn fusionner(a: Vec<Article>, b: Vec<Article>) -> Vec<Article> {
    // TODO 7 : `let mut resultat = a;` (a nous appartient, on peut le rendre
    // mutable), puis `for article in b` (sans & : consomme b, chaque tour
    // reçoit un Article possédé). Même logique que ajouter, mais au lieu de
    // fabriquer un Article, on pousse celui qu'on a reçu : push(article).
    // Aucun .clone() n'est nécessaire.
    todo!("TODO 7")
}

// Les centimes affichés en euros : 4990 donne "49.90". Fournie.
fn euros(centimes: u64) -> String {
    format!("{}.{:02}", centimes / 100, centimes % 100)
}

/// Le tableau de l'inventaire, une ligne par article puis le total :
///
/// ```text
/// Article       Qté      Prix     Total
/// clavier         2     49.90     99.80
/// TOTAL                           99.80
/// ```
///
/// Nom sur 12 colonnes à gauche, puis quantité sur 5, prix sur 10 et total
/// sur 10, à droite. Chaque ligne se termine par un retour à la ligne.
pub fn format_tableau(inv: &[Article]) -> String {
    // TODO 8 : une String mutable, et pour chaque ligne
    // sortie.push_str(&format!("{:<12}{:>5}{:>10}{:>10}\n", ...)).
    // L'en-tête : "Article", "Qté", "Prix", "Total". Les montants avec
    // euros(...). La ligne TOTAL a deux colonnes vides : "".
    let _ = euros(0); // retire cette ligne quand tu utilises euros()
    todo!("TODO 8")
}
