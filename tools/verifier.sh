#!/usr/bin/env bash
# tools/verifier.sh : vérifie que le cours est cohérent :
#   1. tous les liens relatifs des .md pointent vers un fichier existant ;
#   2. aucun tiret cadratin ne traîne dans les textes ;
#   3. chaque labo est formaté (rustfmt), passe clippy sans avertissement
#      (squelette ET solution), son squelette compile avec ses tests, et sa
#      solution passe ses tests (ou se lance, s'il n'y a pas de tests).
# Lancé par : ./tools/verifier.sh depuis la racine du dépôt (Rust 1.85+ requis).
# Variables : LABOS="04-inventaire 11-pinger-parallele" pour n'en vérifier que
# certains ; CARGO_TARGET_DIR pour utiliser un autre dossier de build.
set -u
cd "$(dirname "$0")/.."
erreurs=0

echo "== Liens markdown"
# Extrait chaque [texte](cible) des .md hors blocs de code, ignore les URLs,
# vérifie que la cible existe.
while IFS=: read -r fichier cible; do
    cible="${cible%%#*}"                       # enlève l'ancre #section
    [ -z "$cible" ] && continue
    chemin="$(dirname "$fichier")/$cible"
    if [ ! -e "$chemin" ]; then
        echo "  CASSÉ  $fichier → $cible"
        erreurs=$((erreurs + 1))
    fi
done < <(grep -r --include='*.md' -H '' README.md cours labs tools \
         | awk -F: '{ if ($2 ~ /^```/) { fence[$1] = !fence[$1]; next } if (!fence[$1]) print }' \
         | sed -E 's/`[^`]*`//g' \
         | grep -o -E '^[^:]+:.*\]\([^)]+\)' \
         | sed -E 's/^([^:]+):.*\]\(([^)]+)\)$/\1:\2/' | grep -v -E ':(https?:|mailto:)')
echo "  liens vérifiés"

echo "== Typographie"
if grep -rn --include='*.md' --include='*.rs' --include='*.toml' -e '—' -e '–' README.md cours labs tools >/tmp/verif_rust_tirets.log 2>/dev/null; then
    echo "  tirets cadratins trouvés → /tmp/verif_rust_tirets.log"
    erreurs=$((erreurs + 1))
else
    echo "  aucun tiret cadratin"
fi

echo "== Labos"
for labo in labs/*/; do
    nom=$(basename "$labo")
    [ -n "${LABOS:-}" ] && ! grep -qw "$nom" <<<"$LABOS" && continue
    log=/tmp/verif_rust_$nom.log
    if [ ! -f "$labo/Cargo.toml" ]; then
        echo "  —      $nom (pas de Cargo.toml)"
        continue
    fi
    (
        cd "$labo" || exit 1
        cargo fmt --check || { echo "rustfmt : squelette non formaté"; exit 1; }
        cargo clippy --all-targets --quiet -- -D warnings || exit 1
        cargo test --no-run --quiet || exit 1           # le squelette et ses tests compilent
        cd solution || { echo "pas de dossier solution/"; exit 1; }
        cargo fmt --check || { echo "rustfmt : solution non formatée"; exit 1; }
        cargo clippy --all-targets --quiet -- -D warnings || exit 1
        if [ -d tests ] || grep -rq '#\[test\]' src 2>/dev/null; then
            cargo test --quiet || exit 1
        elif [ -f verifier.sh ]; then
            bash verifier.sh || exit 1
        else
            cargo run --quiet || exit 1
        fi
    ) >"$log" 2>&1
    if [ $? -eq 0 ]; then
        echo "  OK     $nom"
    else
        echo "  ÉCHEC  $nom → $log"
        erreurs=$((erreurs + 1))
    fi
done

echo
if [ "$erreurs" -eq 0 ]; then echo "Tout est bon."; else echo "$erreurs problème(s)."; exit 1; fi
