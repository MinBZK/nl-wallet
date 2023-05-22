ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
# rust 1.69
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:71f9d83281aba918cc47bce4113b1d5d694ca997f3c361795082294fc638f691 AS builder
WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release --package "wallet_provider" --bin wallet_provider

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/wallet_provider .
USER 1000
CMD ["./wallet_provider"]
