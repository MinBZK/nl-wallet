#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./rustup-init https://static.rust-lang.org/rustup/archive/1.29.0/x86_64-unknown-linux-gnu/rustup-init
echo "4acc9acc76d5079515b46346a485974457b5a79893cfb01112423c89aeb5aa10  rustup-init" | sha256sum -c

chmod +x ./rustup-init
./rustup-init -y --default-toolchain 1.94.0 --profile minimal --component clippy,rustfmt
rm ./rustup-init

rustup target add x86_64-unknown-linux-musl x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview # needed by cargo-llvm-cov

cargo --version
rustc --version

cargo install cargo-audit --features=fix --locked --version 0.22.1
cargo install cargo-expand --locked --version 1.0.121
cargo install cargo-llvm-cov --locked --version 0.8.4
cargo install cargo-hack --locked --version 0.6.43
cargo install cargo-nextest --locked --version 0.9.129
cargo install flutter_rust_bridge_codegen --locked --version 2.11.1
cargo install lcov2xml --locked --version 1.0.9
cargo install sea-orm-cli --locked --version 1.1.19 --no-default-features \
    --features sqlx-postgres,runtime-tokio-rustls,cli,codegen
