ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:befd6633cdf0ea52c4e99ae08dd4746eed327e4467d2236df5ddff1f8eb9fc6a AS builder
WORKDIR /usr/src/
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "wallet_server_migration" --bin wallet_server_migration

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/wallet_server_migration .
USER 1000
ENTRYPOINT ["./wallet_server_migration"]
CMD ["status"]
