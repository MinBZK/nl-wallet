ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
# rust 1.73
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:c485e43a3940e33327f97ce24c1ab3c2e67fd5c86c7700df8dee14ad9c0d4a12 AS builder
WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "wallet_server_migration" --bin wallet_server_migration

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/wallet_server_migration .
USER 1000
ENTRYPOINT ["./wallet_server_migration"]
CMD ["status"]
