# Labo 15 : image multi-étapes pour hostinfo.
# Construire : docker build -t hostinfo:1.2.0 .        (ou make docker)
# Lancer     : docker run --rm hostinfo:1.2.0 --json
#
# Étape 1 : la chaîne de compilation complète (près d'un giga-octet), jetée
# ensuite. L'image alpine de Rust compile pour musl par défaut : le binaire
# produit est statique, il ne dépend d'aucune libc et tourne sur scratch.
FROM rust:1.93-alpine AS build
WORKDIR /src

# Les dépendances d'abord, seules, avec des sources vides : cette couche
# (la plus longue, clap et serde à compiler) n'est reconstruite que si
# Cargo.toml change. C'est le principe de cargo-chef, fait à la main.
COPY Cargo.toml ./
COPY .cargo .cargo
RUN mkdir src \
 && echo 'fn main() {}' > src/main.rs \
 && touch src/lib.rs \
 && cargo build --release \
 && rm -rf src

# Puis le vrai code. touch : pour que Cargo voie des sources plus récentes
# que la compilation à vide de l'étape précédente.
COPY src src
RUN touch src/main.rs src/lib.rs \
 && cargo build --release \
 && cp target/release/hostinfo /hostinfo

# Étape 2 : l'image finale. scratch = rien du tout : pas de shell, pas de
# libc, pas de /etc/passwd. Juste le binaire, lancé par un utilisateur non
# root (65534 = nobody, en numérique puisqu'il n'y a pas de /etc/passwd).
FROM scratch
COPY --from=build /hostinfo /hostinfo
USER 65534:65534
ENTRYPOINT ["/hostinfo"]
