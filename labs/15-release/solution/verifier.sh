#!/usr/bin/env bash
# Labo 15, solution : vérifie que hostinfo est livrable.
# Lancé par : (cd solution && bash verifier.sh)
#   1. les tests passent ;
#   2. le binaire release se construit (profil de .cargo/config.toml) ;
#   3. les fichiers de livraison sont là : Dockerfile, Makefile, CI ;
#   4. --version affiche la version écrite dans Cargo.toml.
# Docker n'est pas exigé : le Dockerfile est vérifié par sa présence, pas
# construit (make docker le fait si tu as Docker).
set -euo pipefail
cd "$(dirname "$0")"

echo "== tests"
cargo test --quiet

echo "== build release"
cargo build --release --quiet

echo "== fichiers de livraison"
for f in Dockerfile .dockerignore Makefile rustfmt.toml .cargo/config.toml .github/workflows/ci.yml; do
    [ -f "$f" ] || { echo "  manquant : $f"; exit 1; }
    echo "  ok : $f"
done

echo "== version"
attendue=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
cible=$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
obtenue=$("$cible/release/hostinfo-solution" --version)
if [ "$obtenue" != "hostinfo $attendue" ]; then
    echo "  --version affiche « $obtenue », attendu « hostinfo $attendue »"
    exit 1
fi
echo "  $obtenue"
echo "OK"
