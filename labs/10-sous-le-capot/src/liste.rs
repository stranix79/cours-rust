// Labo 10 : une liste simplement chaînée en Box.
// Lancé par : cargo test liste
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
    tete: Option<Box<Noeud<T>>>,
    len: usize,
}

// Le maillon. Essaie d'écrire `suivant: Option<Noeud<T>>` sans Box et lis
// l'erreur E0072 : un type récursif doit passer par un pointeur.
// Le allow tombe quand push, pop et next lisent ces champs.
#[allow(dead_code)]
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
        // TODO 4a : self.tete.take() sort l'ancienne tête (et laisse None à
        // sa place) ; le nouveau maillon Box::new(Noeud { valeur, suivant:
        // ancienne }) devient self.tete = Some(...). N'oublie pas len.
        let _ = valeur;
        todo!("TODO 4a : push")
    }

    /// Dépile la tête, ou None si la liste est vide.
    pub fn pop(&mut self) -> Option<T> {
        // TODO 4b : self.tete.take().map(|noeud| { ... }) : le suivant du
        // maillon devient la tête, len diminue, et on rend noeud.valeur.
        todo!("TODO 4b : pop sur {} éléments", self.len)
    }

    /// Regarde la tête sans la retirer.
    pub fn peek(&self) -> Option<&T> {
        // TODO 4c : self.tete.as_ref().map(|noeud| &noeud.valeur).
        todo!("TODO 4c : peek sur {} éléments", self.len)
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
        Iter {
            courant: self.tete.as_deref(),
        }
    }
}

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
        // TODO 5 : self.courant.map(|noeud| { ... }) : avance self.courant
        // vers noeud.suivant.as_deref() et rends &noeud.valeur.
        todo!("TODO 5 : next (fin atteinte : {})", self.courant.is_none())
    }
}

// TODO 6 : sans rien écrire, le Drop par défaut libère la tête, qui libère
// son suivant, qui libère son suivant : une récursion par maillon. Quand
// push marchera, le test un_million_de_maillons_sans_exploser_la_pile (plus
// bas) fera planter tout le binaire de tests avec « has overflowed its
// stack ». Ajoute ici un `impl<T> Drop for Liste<T>` dont drop(&mut self)
// détache les maillons un par un dans une boucle :
//     let mut courant = self.tete.take();
//     while let Some(mut noeud) = courant { courant = noeud.suivant.take(); }
// Chaque maillon est libéré à la fin de son tour de boucle, sans suivant.

// Les tests unitaires : ne les modifie pas.
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
