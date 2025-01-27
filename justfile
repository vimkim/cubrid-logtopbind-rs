# Load environment variables from `.env` file.

# show available just recipes
_default:
    @just --list --justfile {{ justfile() }}

audit:
    cargo audit

# build debug executable
build: lint
    cargo build

build-sqlite3-rs:
    cargo build --bin sqlite-rs

build-logtopbind:
    cargo build --bin logtopbind

check:
    cargo check

clean:
    cargo clean

# show dependencies of this project
deps:
    cargo tree

# generate the documentation of this project
docs:
    cargo doc --open

# format source code
format:
    cargo +nightly fmt

# evaluate and print all just variables
just-vars:
    @just --evaluate

# linters (requires https://github.com/rust-lang/rust-clippy)
lint:
    # Default clippy settings (used by `cargo [build, test]` automatically):
    #
    #   cargo clippy
    #
    # If you want stricter clippy settings, start with the suggestion below
    # and consider adding this `lint` target as a dependency to other just
    # targets like `build` and `test`.
    #
    # --all-targets:  check sources and tests
    # --all-features: check non-default crate features
    # -D warnings:    fail the build when encountering warnings
    #
    cargo clippy --verbose --all-targets --all-features -- -D warnings

# detect outdated crates (requires https://github.com/kbknapp/cargo-outdated)
outdated:
    cargo outdated

pre-release: check test lint

# build release executable
release: pre-release
    cargo build --release

queries-db-remove:
    /bin/rm -rf queries.db queries.db-journal

# build and run
run-logtopbind-simple: build-logtopbind queries-db-remove
    ./target/debug/logtopbind ./testdata/log_top.q

run-logtopbind-500k: build-logtopbind queries-db-remove
    ./target/debug/logtopbind ./testdata/log_top_500k.q

run-logtopbind-50m: build-logtopbind queries-db-remove
    ./target/debug/logtopbind ./testdata/log_top_50m.q

# build and run
run-logtopbind-release-simple: queries-db-remove
    cargo run --release --bin logtopbind ./testdata/log_top.q

run-logtopbind-release-500k: queries-db-remove
    cargo run --release --bin logtopbind ./testdata/log_top_500k.q

test: lint
    cargo test

sqlite3:
    sqlite3 queries.db 'select * from logs;'

sqlite3-json:
    sqlite3 queries.db "select bind_statements -> '$[0]' from logs;"

error-clip:
    cargo check |& clip

sqlite3-rs-select: build-sqlite3-rs
    ./target/debug/sqlite-rs queries.db 'select * from logs;'

sqlite3-rs-select-replaced-1: build-sqlite3-rs
    ./target/debug/sqlite-rs queries.db 'select replaced_query from logs limit 1;'

sqlite3-rs-interactive: build-sqlite3-rs
    ./target/debug/sqlite-rs queries.db

build-logtopprint:
    cargo build --bin logtopprint

query-print-3: build-logtopprint
    ./target/debug/logtopprint --query-no 3

# print system information such as OS and architecture
system-info:
    @echo "architecture: {{ arch() }}"
    @echo "os: {{ os() }}"
    @echo "os family: {{ os_family() }}"

release-patch:
    cargo release patch --no-publish
