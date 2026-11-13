#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./rustup-init https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init
echo "b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55  rustup-init" | sha256sum -c

chmod +x ./rustup-init
./rustup-init -y --default-toolchain 1.100.0 --profile minimal --component clippy,rustfmt
rm ./rustup-init

rustup toolchain install nightly --profile minimal --component rustfmt
rustup target add x86_64-unknown-linux-musl x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview # needed by cargo-llvm-cov

cargo --version
rustc --version

cargo install cargo-audit --features=fix --version 0.23.0
cargo install cargo-expand --version 1.0.127
cargo install cargo-llvm-cov --version 0.9.1
cargo install cargo-hack --version 0.6.46
cargo install cargo-nextest --version 0.9.151
cargo install flutter_rust_bridge_codegen --version 2.12.0
cargo install lcov2xml --version 1.0.10
cargo install sea-orm-cli --version 1.1.19 --no-default-features \
    --features sqlx-postgres,runtime-tokio-rustls,cli,codegen
