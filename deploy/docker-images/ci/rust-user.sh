#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./rustup-init https://static.rust-lang.org/rustup/archive/1.28.2/x86_64-unknown-linux-gnu/rustup-init
echo "20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c  rustup-init" | sha256sum -c

chmod +x ./rustup-init
./rustup-init -y --default-toolchain 1.88.0 --profile minimal --component clippy,rustfmt
rm ./rustup-init

rustup target add x86_64-unknown-linux-musl x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview # needed by cargo-llvm-cov

cargo --version
rustc --version

cargo install cargo-expand --locked --version 1.0.113
cargo install cargo-llvm-cov --locked --version 0.6.17
cargo install cargo-audit --locked --version 0.21.2
cargo install cargo-nextest --locked --version 0.9.101
cargo install flutter_rust_bridge_codegen --locked --version 2.11.1
