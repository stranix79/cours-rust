-- Migration 001 : la table des articles.
-- Le même SQL tourne sur SQLite et sur PostgreSQL : BIGINT est un entier
-- 64 bits pour PostgreSQL et un INTEGER pour SQLite, et la contrainte CHECK
-- refuse un stock négatif quel que soit le programme qui écrit dans la base.
-- IF NOT EXISTS : rejouer la migration au démarrage ne casse rien.
CREATE TABLE IF NOT EXISTS articles (
    nom      TEXT PRIMARY KEY,
    quantite BIGINT NOT NULL CHECK (quantite >= 0)
);
