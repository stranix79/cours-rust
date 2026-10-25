// Labo 04, solution : un inventaire d'articles, pour faire travailler la
// possession et les emprunts.
// Testé par : (cd solution && cargo test)   Lancé par : (cd solution && cargo run)
//
// Chaque signature de ce fichier répond à une seule question : qui possède
// quoi ? Lis-les avant le corps des fonctions. La règle :
//   - `T`       : la fonction prend possession (l'appelant ne l'a plus) ;
//   - `&T`      : la fonction lit, l'appelant garde la valeur ;
//   - `&mut T`  : la fonction modifie, l'appelant garde la valeur, et
//                 personne d'autre n'y touche pendant l'appel.

/// Un article en stock.
// Debug pour {:?} et assert_eq!, Clone pour pouvoir dupliquer un article
// explicitement (.clone()), PartialEq pour comparer avec ==. Pas de Copy :
// un article contient une String, qui possède un bloc sur le tas, et un tel
// type ne peut pas être copié octet par octet (chapitre 4.3).
#[derive(Debug, Clone, PartialEq)]
pub struct Article {
    /// Le nom, possédé par l'article : une String, pas une &str. Un article
    /// qui empruntait son nom ne pourrait pas vivre plus longtemps que le
    /// texte d'où vient ce nom, ce qui rendrait l'inventaire inutilisable.
    pub nom: String,
    pub quantite: u32,
    /// Le prix unitaire en centimes : de l'argent dans un entier, jamais
    /// dans un f64 (0.1 + 0.2 != 0.3).
    pub prix_centimes: u64,
}

/// Ajoute `quantite` articles. Si un article du même nom existe déjà, sa
/// quantité augmente et son prix ne change pas ; sinon l'article est créé.
// `inv: &mut Vec<Article>` : on modifie le Vec de l'appelant (on peut y
// pousser un élément), donc un emprunt mutable du Vec lui-même. Une
// &mut [Article] ne suffirait pas : une tranche ne peut pas grandir.
// `nom: &str` : on ne fait que lire le nom, et on n'en fabrique une String
// que si l'article est nouveau. L'appelant peut passer "clavier" comme
// &String sans rien céder.
pub fn ajouter(inv: &mut Vec<Article>, nom: &str, quantite: u32, prix_centimes: u64) {
    // iter_mut() parcourt le Vec en donnant des &mut Article : on peut
    // modifier chaque élément sur place. find s'arrête au premier qui
    // correspond et rend Option<&mut Article>.
    // `a.nom == nom` compare une String et une &str : PartialEq existe
    // entre les deux, pas besoin de convertir.
    if let Some(article) = inv.iter_mut().find(|a| a.nom == nom) {
        article.quantite += quantite;
    } else {
        // Dans le else, l'emprunt mutable pris par iter_mut() est déjà
        // terminé (on ne se sert plus de `article`), donc on peut emprunter
        // `inv` à nouveau pour push. to_string() alloue la String que
        // l'article possédera.
        inv.push(Article {
            nom: nom.to_string(),
            quantite,
            prix_centimes,
        });
    }
}

/// Cherche un article par son nom.
// La durée de vie 'a dit au compilateur, et au lecteur : la référence
// rendue pointe DANS `inv`, pas dans `nom`. Avec deux références en entrée,
// les règles d'élision ne savent pas choisir, il faut l'écrire. Grâce à ça,
// l'appelant peut libérer la chaîne `nom` aussitôt après l'appel et garder
// le résultat, tant que l'inventaire vit.
pub fn chercher<'a>(inv: &'a [Article], nom: &str) -> Option<&'a Article> {
    // iter() donne des &Article ; find rend le premier, ou None. Aucune
    // copie : on rend une référence vers l'élément qui est dans le Vec.
    inv.iter().find(|a| a.nom == nom)
}

/// Retire `quantite` articles et rend la quantité restante. Un article qui
/// tombe à zéro disparaît de l'inventaire. Erreur si l'article est inconnu
/// ou si le stock ne suffit pas (et alors rien n'est modifié).
// &mut Vec<Article> encore : on peut supprimer un élément, ce qui change la
// longueur du Vec.
pub fn retirer(inv: &mut Vec<Article>, nom: &str, quantite: u32) -> Result<u32, String> {
    // On cherche un INDICE plutôt qu'une référence. Une &mut Article prise
    // avec iter_mut().find(), dont on se servirait encore après un
    // inv.remove(), serait refusée : `inv` emprunté en mutable deux fois à
    // la fois (E0499). Un usize, lui, n'emprunte rien. position() rend
    // Option<usize>.
    let Some(i) = inv.iter().position(|a| a.nom == nom) else {
        return Err(format!("article inconnu : {nom}"));
    };
    // inv[i] : accès par indice, vérifié à l'exécution. Ici i vient de
    // position(), il est forcément valide.
    let disponible = inv[i].quantite;
    if quantite > disponible {
        return Err(format!(
            "stock insuffisant pour {nom} : {disponible} en stock, {quantite} demandés"
        ));
    }
    // Pas de débordement possible : on vient de vérifier quantite <= disponible.
    let reste = disponible - quantite;
    if reste == 0 {
        // remove(i) décale les éléments suivants pour boucher le trou et
        // rend l'article retiré, que l'on jette : il est libéré ici, sa
        // String avec lui (Drop).
        inv.remove(i);
    } else {
        inv[i].quantite = reste;
    }
    Ok(reste)
}

/// La valeur du stock : somme des quantités multipliées par les prix.
// &[Article] : une tranche empruntée. La fonction lit, et accepte un
// &Vec<Article> comme un morceau de Vec (&inv[1..]) ou un tableau.
// Demander &Vec<Article> serait plus restrictif sans rien apporter
// (clippy le signale : ptr_arg).
pub fn valeur_totale(inv: &[Article]) -> u64 {
    let mut total = 0;
    // `for article in inv` sur une &[Article] donne des &Article : on lit
    // chaque article sans le sortir du Vec.
    for article in inv {
        // u64::from(u32) : conversion sans perte, donc From et pas `as`.
        total += u64::from(article.quantite) * article.prix_centimes;
    }
    total
}

/// L'article au prix unitaire le plus élevé, ou None si l'inventaire est
/// vide. En cas d'égalité, le dernier rencontré.
// Une seule référence en entrée, une en sortie : l'élision s'applique, le
// compilateur comprend tout seul que la sortie emprunte à `inv`. Écrit en
// entier, ce serait fn plus_cher<'a>(inv: &'a [Article]) -> Option<&'a Article>.
pub fn plus_cher(inv: &[Article]) -> Option<&Article> {
    // max_by_key compare les clés (ici le prix, un u64 Copy) et rend une
    // référence vers l'élément gagnant.
    inv.iter().max_by_key(|a| a.prix_centimes)
}

/// Les noms des articles, dans l'ordre de l'inventaire.
// Vec<&str> : des vues sur les String des articles, sans aucune copie de
// texte. L'élision donne à chaque &str la durée de vie de `inv` : la règle
// est la même que pour plus_cher (une seule référence en entrée, elle est
// prêtée à toutes les références de la sortie). Conséquence : tant que ce
// Vec<&str> sert, l'inventaire ne peut pas être modifié (E0502), et c'est
// heureux, car un push pourrait déplacer les articles en mémoire.
pub fn noms(inv: &[Article]) -> Vec<&str> {
    // with_capacity : on connaît la taille finale, une seule allocation.
    let mut resultat = Vec::with_capacity(inv.len());
    for article in inv {
        // as_str() : une &str qui regarde la String de l'article. &article.nom
        // marcherait aussi (une &String se convertit toute seule en &str).
        resultat.push(article.nom.as_str());
    }
    resultat
}

/// Fusionne deux inventaires en un seul : les articles de `b` s'ajoutent à
/// ceux de `a`, les quantités des noms communs s'additionnent (le prix de
/// `a` est gardé). L'ordre est celui de `a`, puis les nouveaux de `b`.
// `a: Vec<Article>, b: Vec<Article>` sans & : la fonction PREND les deux
// inventaires. L'appelant ne pourra plus s'en servir après l'appel (E0382
// s'il essaie), et en échange on peut déplacer les articles de b dans a
// sans cloner une seule String.
pub fn fusionner(a: Vec<Article>, b: Vec<Article>) -> Vec<Article> {
    // `a` nous appartient : on peut le rendre mutable, même si l'appelant
    // l'avait déclaré sans mut. La mutabilité suit le propriétaire.
    let mut resultat = a;
    // `for article in b` (sans &) consomme b : chaque tour reçoit un
    // Article possédé, sorti du Vec. Après la boucle, b n'existe plus.
    for article in b {
        if let Some(existant) = resultat.iter_mut().find(|x| x.nom == article.nom) {
            existant.quantite += article.quantite;
            // `article` est libéré à la fin de ce tour : son nom ne sert plus.
        } else {
            // Déplacement : l'article (et sa String) passe dans resultat,
            // aucune copie du texte.
            resultat.push(article);
        }
    }
    resultat
}

// Les centimes affichés en euros : 4990 donne "49.90". Une fonction privée
// (pas de pub), utilisée par format_tableau. {:02} complète à deux chiffres.
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
    // Une String que l'on remplit : push_str ajoute une &str à la fin,
    // en agrandissant le tampon si besoin.
    let mut sortie = String::new();
    // La largeur compte des caractères, pas des octets : "Qté" occupe bien
    // 3 colonnes, même si le é fait 2 octets en UTF-8.
    sortie.push_str(&format!(
        "{:<12}{:>5}{:>10}{:>10}\n",
        "Article", "Qté", "Prix", "Total"
    ));
    for a in inv {
        let total = u64::from(a.quantite) * a.prix_centimes;
        // &format!(...) : format! rend une String temporaire, push_str veut
        // une &str, on lui prête la String le temps de la copier.
        sortie.push_str(&format!(
            "{:<12}{:>5}{:>10}{:>10}\n",
            a.nom,
            a.quantite,
            euros(a.prix_centimes),
            euros(total)
        ));
    }
    sortie.push_str(&format!(
        "{:<12}{:>5}{:>10}{:>10}\n",
        "TOTAL",
        "",
        "",
        euros(valeur_totale(inv))
    ));
    sortie
}
