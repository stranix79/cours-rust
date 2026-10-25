// Labo 10, solution : une liste simplement chaînée en Box.
// Lancé par : (cd solution && cargo test liste)
//
// La structure de données qu'on écrit en C en premier, et qu'on n'écrit
// presque jamais en Rust (Vec fait mieux dans 99 % des cas). On l'écrit ici
// parce qu'elle montre trois choses : pourquoi un type récursif a besoin de
// Box, comment Option::take déplace une valeur hors d'un &mut, et pourquoi
// le Drop par défaut d'une longue chaîne fait exploser la pile.

//! Une liste simplement chaînée en `Option<Box<Noeud<T>>>`, avec un `Drop` itératif.

/// Une pile LIFO en liste chaînée : `push` et `pop` en tête, en O(1).
#[derive(Debug)]
pub struct Liste<T> {
    // Option<Box<…>> : None marque la fin. Grâce à la niche (chapitre 10),
    // ce champ fait la taille d'un seul pointeur, comme le NULL du C.
    tete: Option<Box<Noeud<T>>>,
    len: usize,
}

// Le maillon. Sans Box, `suivant: Option<Noeud<T>>` donnerait un type de
// taille infinie (E0072 : recursive type has infinite size) : un Noeud
// contiendrait un Noeud qui contiendrait un Noeud... Box met le suivant sur
// le tas et ne garde ici qu'un pointeur, de taille connue.
#[derive(Debug)]
struct Noeud<T> {
    valeur: T,
    suivant: Option<Box<Noeud<T>>>,
}

impl<T> Liste<T> {
    /// Une liste vide. Aucune allocation.
    pub fn new() -> Self {
        Liste { tete: None, len: 0 }
    }

    /// Empile `valeur` en tête.
    pub fn push(&mut self, valeur: T) {
        // take() remplace self.tete par None et rend l'ancienne valeur : on
        // ne peut pas « sortir » un champ d'un &mut sans y laisser quelque
        // chose, take() y laisse None. Le nouveau nœud prend l'ancienne tête
        // comme suivant, puis devient la tête.
        let ancien = self.tete.take();
        self.tete = Some(Box::new(Noeud {
            valeur,
            suivant: ancien,
        }));
        self.len += 1;
    }

    /// Dépile la tête, ou None si la liste est vide.
    pub fn pop(&mut self) -> Option<T> {
        // map sur l'Option : si une tête existe, on la déballe (le Box est
        // déplacé dans `noeud`), son suivant devient la tête, et on rend la
        // valeur. Le Box vide est libéré à la fin de la fermeture.
        self.tete.take().map(|noeud| {
            self.tete = noeud.suivant;
            self.len -= 1;
            noeud.valeur
        })
    }

    /// Regarde la tête sans la retirer.
    pub fn peek(&self) -> Option<&T> {
        // as_ref : &Option<Box<Noeud>> → Option<&Box<Noeud>>, sans déplacer.
        self.tete.as_ref().map(|noeud| &noeud.valeur)
    }

    /// Le nombre d'éléments, tenu à jour par push et pop : O(1).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Vrai si la liste est vide (clippy demande is_empty à côté de len).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Un itérateur par référence, de la tête vers la queue.
    pub fn iter(&self) -> Iter<'_, T> {
        // as_deref : Option<Box<Noeud>> vu comme Option<&Noeud>. L'itérateur
        // emprunte la liste : tant qu'il vit, pas de push possible.
        Iter {
            courant: self.tete.as_deref(),
        }
    }
}

// Default : Liste::default() et #[derive(Default)] sur les types qui en
// contiennent une. Clippy le réclame quand new() ne prend rien.
impl<T> Default for Liste<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// L'itérateur de [`Liste::iter`]. La durée de vie `'a` dit que les `&T`
/// rendus ne peuvent pas survivre à la liste.
pub struct Iter<'a, T> {
    courant: Option<&'a Noeud<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        // On avance d'un maillon et on rend la valeur de celui qu'on quitte.
        // Option<&Noeud> est Copy : map ne déplace rien d'important.
        self.courant.map(|noeud| {
            self.courant = noeud.suivant.as_deref();
            &noeud.valeur
        })
    }
}

// Le Drop par défaut libérerait la tête, qui libérerait son suivant, qui
// libérerait son suivant... une récursion par maillon. Un million de
// maillons, c'est un million de cadres de pile : « thread has overflowed
// its stack ». On le remplace par une boucle : chaque maillon est détaché
// (take) AVANT d'être libéré, donc sa libération ne déclenche rien d'autre.
impl<T> Drop for Liste<T> {
    fn drop(&mut self) {
        let mut courant = self.tete.take();
        // while let : tant qu'il reste un maillon, on prend son suivant, puis
        // le maillon (dont suivant vaut maintenant None) est libéré à la fin
        // de l'itération, sans récursion.
        while let Some(mut noeud) = courant {
            courant = noeud.suivant.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_million_de_maillons_sans_exploser_la_pile() {
        // Sans le Drop itératif, ce test fait planter TOUT le binaire de
        // tests avec « has overflowed its stack » : un test lancé dans un
        // thread de 2 Mio de pile ne survit pas à un million d'appels.
        let mut l = Liste::new();
        for i in 0..1_000_000 {
            l.push(i);
        }
        assert_eq!(l.len(), 1_000_000);
        drop(l); // c'est ici que tout se joue
    }

    #[test]
    fn la_tete_est_privee_mais_visible_ici() {
        let mut l = Liste::new();
        l.push("a");
        // Un module de tests enfant voit les champs privés.
        assert!(l.tete.is_some());
        assert!(l.tete.as_ref().unwrap().suivant.is_none());
    }
}
