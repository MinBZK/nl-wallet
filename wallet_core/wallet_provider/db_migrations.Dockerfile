ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
# rust 1.69
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:534ca807364ac75d02d348f33faa9f1455f9ba7a48af52790811d717b57be001 AS builder
WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "wallet_provider" --bin wallet_provider_migrations --features wallet_provider_migrations

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/wallet_provider_migrations .
USER 1000
ENTRYPOINT ["./wallet_provider_migrations"]
CMD ["status"]