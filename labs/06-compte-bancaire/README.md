# Labo 06 : Compte bancaire

**Chapitre** : [6. Structs, enums, méthodes, Option](../../cours/06-structs-enums-option.md)

## Objectif
Modéliser une petite banque avec ce que le chapitre 6 apporte : un enum à données pour l'historique (`Operation`), un enum d'erreurs avec `Display` (`ErreurBanque`), une struct à champs privés qui protège son solde (`Compte`), des méthodes à `&self` et `&mut self`, et des `Option` partout où une valeur peut manquer. Le morceau de bravoure est `virer` : modifier deux comptes rangés dans la même `HashMap`, donc tenir deux `&mut` en même temps, ce que le vérificateur d'emprunts refuse d'abord. À la fin, tu sais choisir un receveur, écrire un `match` exhaustif sur tes propres enums, et contourner proprement un E0499.

## Consignes
1. Lis `src/lib.rs` : les types sont écrits, ainsi que `Compte::new`, les trois accesseurs et `Banque::new`. Les autres méthodes ont un corps `todo!("TODO n")`. Lance `cargo test` : les onze tests échouent, mais ça compile.
2. `euros(centimes) -> String` (TODO 1) : `1250` donne `12,50 €`, `-5` donne `-0,05 €`. Le piège est le signe des petits montants négatifs. `cargo test euros`.
3. `Display` pour `ErreurBanque` (TODO 2) : les quatre messages exacts sont dans le test. Une fois écrit, `to_string()` marche tout seul. `cargo test messages`.
4. `deposer(&mut self, montant)` (TODO 3) et `retirer(&mut self, montant)` (TODO 4) : un montant nul ou négatif est refusé, un retrait ne dépasse pas le solde, et un refus ne laisse **aucune trace** (ni dans le solde, ni dans l'historique). Écris la méthode privée `verifier_retrait`, elle resservira. `cargo test deposer` puis `cargo test retirer`.
5. `ouvrir(&mut self, titulaire) -> &mut Compte` (TODO 5) et `compte(&self, titulaire) -> Option<&Compte>` (TODO 6) : `entry().or_insert_with()` et `get`. Rouvrir un compte existant ne le remet pas à zéro. `cargo test ouvrir`.
6. `virer(&mut self, de, vers, montant)` (TODO 7) : écris d'abord la version naïve, avec deux `get_mut` gardés en même temps, et lis l'erreur (ici sur une version réduite, où les comptes ne sont que des `i64`) :
   ```
   error[E0499]: cannot borrow `self.comptes` as mutable more than once at a time
     --> src/main.rs:10:21
      |
    9 |         let source = self.comptes.get_mut(de)?;
      |                      ------------ first mutable borrow occurs here
   10 |         let cible = self.comptes.get_mut(vers)?;
      |                     ^^^^^^^^^^^^ second mutable borrow occurs here
   ```
   Le compilateur ne sait pas que `de` et `vers` sont différents ; s'ils étaient égaux, tu aurais deux `&mut` sur le même compte, exactement ce que le chapitre 4 interdit. Le commentaire du TODO donne les deux sorties propres. Vérifie tout avant de modifier quoi que ce soit : le test `virer_refuse_et_ne_change_rien` contrôle que chaque refus laisse les deux comptes intacts, et qu'un virement vers un inconnu n'ouvre pas de compte. `cargo test virer`.
7. `libelle` et `montant_signe` sur `Operation` (TODO 8), puis `releve(&self, titulaire) -> Option<String>` (TODO 9) : le format est comparé caractère par caractère, espaces compris. `{:<24}` et `{:>12}` comptent en caractères, pas en octets : `€` compte pour un. `cargo test releve`.
8. `cargo run` : compare avec la sortie attendue. Puis `cargo test`, `cargo clippy --all-targets` et `cargo fmt --check` : rien à redire.

## Comment lancer
```bash
cargo test                       # tes tests (rouge tant que les TODO restent)
cargo test virer                 # seulement les tests dont le nom contient virer
cargo run                        # ton programme : le scénario de main.rs
(cd solution && cargo test)      # la solution (vert)
(cd solution && cargo run)
cargo clippy --all-targets && cargo fmt --check
```

## Sortie attendue
```
$ cargo run
alice et bob ouvrent leur compte
virement alice -> bob de 30,00 € : ok
virement bob -> alice refusé : solde insuffisant : 50,00 € disponibles, 90,00 € demandés
virement alice -> carol refusé : compte inconnu : carol
virement alice -> alice refusé : virement vers le même compte
virement bob -> alice refusé : montant invalide : -0,10 € (il doit être positif)
retrait bob refusé : solde insuffisant : 50,00 € disponibles, 1000,00 € demandés

Relevé de alice
  dépôt                      +100,00 €
  virement vers bob           -30,00 €
  retrait                     -12,50 €
Solde : 57,50 €
Relevé de bob
  dépôt                       +20,00 €
  virement de alice           +30,00 €
Solde : 50,00 €
pas de compte pour carol
```

## Pour aller plus loin
- Ajoute un découvert autorisé par compte (`decouvert_centimes`, privé, avec une méthode pour le fixer) : seul `verifier_retrait` change, et c'est la preuve que la factoriser valait le coup.
- Écris `Banque::fermer(&mut self, titulaire) -> Result<Compte, ErreurBanque>` qui sort le compte de la map avec `remove` et le rend à l'appelant par valeur. Pourquoi faut-il refuser de fermer un compte dont le solde n'est pas nul, et quelle variante d'erreur ajouter ?
- `ErreurBanque` a `Display` mais pas le trait `std::error::Error` : impossible de la renvoyer depuis un `main` qui rend `Result<(), Box<dyn Error>>`. Le chapitre 7 explique les traits, le chapitre 8 ajoute cette implémentation en une ligne, puis la fait écrire par `thiserror`.
