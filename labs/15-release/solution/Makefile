# Labo 15, solution : Makefile de hostinfo (binaire hostinfo-solution).
# Usage : make build | run | test | lint | release | cross | docker | clean
#         make cross CIBLES="aarch64-unknown-linux-musl"   (une seule cible)
#
# Cargo fait déjà tout ; le Makefile ne décrit pas la compilation, il retient
# les commandes et leurs options pour que la CI, toi et tes collègues lanciez
# exactement les mêmes.

BIN     := hostinfo-solution
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
# Le dossier target/ : celui de l'espace de travail du cours (../../target),
# ou CARGO_TARGET_DIR s'il est défini. cargo metadata le sait mieux que nous.
TARGET  := $(shell cargo metadata --format-version 1 --no-deps 2>/dev/null \
             | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
CIBLES  ?= x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-gnu

.PHONY: build run test lint release cross docker clean

## build : binaire de debug, pour développer
build:
	cargo build

## run : lance le programme
run:
	cargo run -- $(ARGS)

## test : les tests, comme en CI
test:
	cargo test

## lint : formatage, puis clippy avec les règles de [lints] en erreurs
lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

## release : binaire optimisé (profil de .cargo/config.toml), et sa taille
release:
	cargo build --release
	@ls -l $(TARGET)/release/$(BIN) | awk '{ print $$5, "octets :", $$9 }'

## cross : un binaire par cible. Pour du Rust pur, rust-lld (livré avec
## rustup) suffit à lier les cibles musl depuis un Mac ; avec du code C dans
## les dépendances, remplace par CARGO="cargo zigbuild" ou CARGO=cross.
CARGO ?= cargo
cross:
	@for c in $(CIBLES); do \
		rustup target add $$c >/dev/null 2>&1; \
		echo "  $$c"; \
		case $$c in \
			*-linux-musl) lieur="CARGO_TARGET_$$(echo $$c | tr a-z- A-Z_)_LINKER=rust-lld" ;; \
			*) lieur="" ;; \
		esac; \
		env $$lieur $(CARGO) build --release --target $$c || exit 1; \
	done

## docker : image multi-étapes, binaire statique seul sur scratch
docker:
	docker build -t hostinfo:$(VERSION) .
	docker images hostinfo

clean:
	cargo clean -p lab15-release-solution
