# Labo 16 : image multi-étapes de sondes.
# Construire ton code : docker build -t sondes:dev .
# Ou la solution     : docker build --build-arg SRC=solution --build-arg BIN=sondes-solution -t sondes:dev .
# Lancer             : docker run --rm -p 8080:8080 -v sondes-data:/data sondes:dev
#
# Étape 1 : compiler. L'image alpine de Rust compile pour musl : binaire
# statique, qui tourne sur scratch. rusqlite (feature bundled) compile le C
# de SQLite : il faut un compilateur C et les en-têtes de musl, d'où musl-dev.
FROM rust:1.93-alpine AS build
RUN apk add --no-cache musl-dev
ARG SRC=.
ARG BIN=sondes
WORKDIR /src

# Les dépendances d'abord, avec des sources vides : tokio, axum, rusqlite et
# SQLite (une minute de compilation) restent en cache tant que Cargo.toml ne
# change pas. C'est ce que cargo-chef automatise.
COPY ${SRC}/Cargo.toml ./
RUN mkdir src \
 && echo 'fn main() {}' > src/main.rs \
 && touch src/lib.rs \
 && cargo build --release \
 && rm -rf src

# Puis le vrai code, le schéma SQL, et la compilation de notre crate seule.
COPY ${SRC}/src src
COPY ${SRC}/migrations migrations
RUN touch src/main.rs src/lib.rs \
 && cargo build --release \
 && cp target/release/${BIN} /sondes \
 # Le dossier de la base, à l'utilisateur qui fera tourner le service.
 && mkdir -p /data \
 && chown 65534:65534 /data

# Étape 2 : l'image finale. Rien d'autre que le binaire, la configuration et
# un dossier /data où l'utilisateur 65534 (nobody) a le droit d'écrire.
FROM scratch
COPY --from=build /sondes /sondes
COPY sondes.toml /etc/sondes/sondes.toml
COPY --from=build --chown=65534:65534 /data /data
USER 65534:65534
# Dans un conteneur, on écoute sur toutes les interfaces : 127.0.0.1 ne
# serait joignable que depuis le conteneur lui-même.
ENV SONDES_CONFIG=/etc/sondes/sondes.toml \
    SONDES_ECOUTE=0.0.0.0:8080 \
    SONDES_BASE=/data/sondes.db
EXPOSE 8080
VOLUME /data
ENTRYPOINT ["/sondes"]
