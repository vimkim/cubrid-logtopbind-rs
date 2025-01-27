# Load environment variables from `.env` file.

set dotenv-load := true

# Extracts the binary name from the settings `name = <...>` in the `[[bin]]`
# section of Cargo.toml
#
# Alternative command:
# `sed -n '/[[bin]]/,/name =/p' Cargo.toml | awk '/^name =/{gsub(/"/, "", $3); print $3}'`

binary := `sed -n '/[[bin]]/,/name =/p' Cargo.toml | awk '/^name =/{gsub(/"/, "", $3); print $3}'`

# Get version from Cargo.toml/Cargo.lock
#
# Alternative command:
# `cargo metadata --format-version=1 | jq '.packages[]|select(.name=="rust-template").version'`

version := `cargo pkgid | sed -rn s'/^.*#(.*)$/\1/p'`
project_dir := justfile_directory()

# show available just recipes
default:
    @just --list --justfile {{ justfile() }}

# show Assembly, LLVM-IR, MIR and WASM for Rust code (requires https://github.com/pacak/cargo-show-asm)
asm *args='':
    # Examples:
    # just asm --lib
    # just asm --lib 0
    # just asm --lib "rust_template::doubler"
    # just asm --bin rust-template-app 0
    cargo asm {{ args }}

# detect known vulnerabilities (requires https://github.com/rustsec/rustsec)
audit:
    cargo audit

# list the biggest functions in the release build (requires https://github.com/RazrFalcon/cargo-bloat)
bloat-biggest-functions:
    cargo bloat --release -n 10

# list the biggest dependencies in the release build (requires https://github.com/RazrFalcon/cargo-bloat)
bloat-biggest-deps:
    cargo bloat --release --crates

# generate report for compilation times
timings:
    @echo "Details at https://doc.rust-lang.org/stable/cargo/reference/timings.html"
    cargo build --timings

# build debug executable
build: lint
    cargo build && echo "Executable at target/debug/{{ binary }}"

build-sqlite3-rs:
    cargo build --bin sqlite-rs

build-logtopbind:
    cargo build --bin logtopbind

# analyze the current package and report errors, but don't build object files (faster than 'build')
check:
    cargo check

# remove generated artifacts
clean:
    cargo clean

# show test coverage (requires https://lib.rs/crates/cargo-llvm-cov)
coverage:
    cargo llvm-cov nextest --open

# show dependencies of this project
deps:
    cargo tree

# create a docker image (requires Docker); run with SHOW_PROGRESS=1 to enable verbose output
docker-image-create $SHOW_PROGRESS="0":
    @echo "Creating a docker image ... (add SHOW_PROGRESS=1 to just command to enable verbose output)"
    ./tools/docker/create_image.sh

# size of the docker image (requires Docker)
docker-image-size:
    docker images $DOCKER_IMAGE_NAME

# run the docker image (requires Docker)
docker-image-run:
    @echo "Running container from docker image ..."
    ./tools/docker/start_container.sh

# generate the documentation of this project
docs:
    cargo doc --open

# format source code
format:
    cargo +nightly fmt

# build and install the binary locally
install: build test
    cargo install --path .

# build and install the static binary locally
install-static: build test
    RUSTFLAGS='-C target-feature=+crt-static' cargo install --path .

# evaluate and print all just variables
just-vars:
    @just --evaluate

# Show license of dependencies (requires https://github.com/onur/cargo-license)
license:
    cargo license

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

# detect undefined behavior with miri (requires https://github.com/rust-lang/miri)
miri:
    cargo clean
    cargo +nightly miri test
    cargo +nightly miri run

# detect outdated crates (requires https://github.com/kbknapp/cargo-outdated)
outdated:
    cargo outdated

# check, test, lint, miri
pre-release: check test lint audit miri

# profile the release binary (requires https://github.com/mstange/samply, which uses profiler.firefox.com as UI)
profile-release:
    # Requires a profile named 'profiling' in ~/.cargo/config.toml
    cargo build --profile profiling && \
    RUST_BACKTRACE=1 samply record target/profiling/{{ binary }}

# build release executable
release: pre-release
    cargo build --release && echo "Executable at target/release/{{ binary }}"

# build and run
run-logtopbind-simple: build-logtopbind
    /bin/rm -rf queries.db
    ./target/debug/logtopbind ./testdata/log_top.q

# build and run
run-logtopbind-release-simple:
    /bin/rm -rf queries.db
    cargo run --release --bin logtopbind ./testdata/log_top.q

run-logtopbind-500k: build-logtopbind
    /bin/rm -rf queries.db
    ./target/debug/logtopbind ./testdata/log_top_500k.q

run-logtopbind-release-500k:
    /bin/rm -rf queries.db
    cargo run --release --bin logtopbind ./testdata/log_top_500k.q

# print system information such as OS and architecture
system-info:
    @echo "architecture: {{ arch() }}"
    @echo "os: {{ os() }}"
    @echo "os family: {{ os_family() }}"

# run tests (requires https://nexte.st/)
test: lint
    cargo nextest run

# run tests in vanilla mode (use when nextest is not installed)
test-vanilla: lint
    cargo test

# show version of this project
version:
    @echo "{{ version }}"

# test a debug binary with valgrind (requires valgrind; supported on Linux, but e.g., not on macOS)
[linux]
valgrind: clean build
    valgrind -v --error-exitcode=1 --track-origins=yes --leak-check=full target/debug/rust-template-app

# run build when sources change (requires https://github.com/watchexec/watchexec)
watch:
    # Watch all rs and toml files in the current directory and all
    # subdirectories for changes.  If something changed, re-run the build.
    @watchexec --clear --exts rs,toml -- just build

# run check then tests when sources change (requires https://github.com/watchexec/cargo-watch)
watch-test:
    cargo watch -q -c -x check -x 'nextest run'

# run tests when sources change (requires https://github.com/Canop/bacon)
watch-test-bacon:
    bacon --no-wrap test

sqlite3:
    sqlite3 queries.db 'select * from log_entries;'

sqlite3-json:
    sqlite3 queries.db "select bind_statements -> '$[0]' from log_entries;"

error-clip:
    cargo check |& clip

sqlite3-rs-select: build-sqlite3-rs
    ./target/debug/sqlite-rs queries.db 'select * from log_entries;'

sqlite3-rs-select-replaced-1: build-sqlite3-rs
    ./target/debug/sqlite-rs queries.db 'select replaced_query from log_entries limit 1;'

sqlite3-rs-interactive: build-sqlite3-rs
    ./target/debug/sqlite-rs queries.db
