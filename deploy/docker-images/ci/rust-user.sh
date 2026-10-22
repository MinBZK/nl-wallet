#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./rustup-init https://static.rust-lang.org/rustup/archive/1.29.2/x86_64-unknown-linux-gnu/rustup-init
echo "5be191e8c70a5007074254cb74d78dbfdd25cd5a61c27de7b8e5410122ec24f4  rustup-init" | sha256sum -c

chmod +x ./rustup-init
./rustup-init -y --default-toolchain 1.98.0 --profile minimal --component clippy,rustfmt
rm ./rustup-init

rustup toolchain install nightly --profile minimal --component rustfmt
rustup target add x86_64-unknown-linux-musl x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview # needed by cargo-llvm-cov

cargo --version
rustc --version

cargo install cargo-audit --features=fix --locked --version 0.22.2
cargo install cargo-expand --locked --version 1.0.124
cargo install cargo-llvm-cov --locked --version 0.8.7
cargo install cargo-hack --locked --version 0.6.45
cargo install cargo-nextest --locked --version 0.9.143
cargo install flutter_rust_bridge_codegen --locked --version 2.12.0
cargo install lcov2xml --locked --version 1.0.9
cargo install sea-orm-cli --locked --version 1.1.19 --no-default-features \
    --features sqlx-postgres,runtime-tokio-rustls,cli,codegen
