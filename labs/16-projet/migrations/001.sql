-- Migration 001 : la table des mesures.
-- Une ligne par sonde : quelle cible, quand (secondes depuis l'époque Unix),
-- si elle a répondu, en combien de millisecondes, et l'erreur sinon.
-- L'index (cible, instant) sert les deux lectures du service : les dernières
-- mesures d'une cible, et le taux de disponibilité depuis un instant.
CREATE TABLE IF NOT EXISTS mesures (
    id       INTEGER PRIMARY KEY,
    cible    TEXT    NOT NULL,
    instant  INTEGER NOT NULL,
    ok       INTEGER NOT NULL CHECK (ok IN (0, 1)),
    duree_ms INTEGER NOT NULL,
    erreur   TEXT
);
CREATE INDEX IF NOT EXISTS mesures_cible_instant ON mesures (cible, instant);
