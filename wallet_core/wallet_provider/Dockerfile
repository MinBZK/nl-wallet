ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

### Build Stage ###
# rust 1.73
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:cfa7e4a056304ebcfaae54d0b40916c884cd7fea839b447314aa89095c7dde4c AS builder
WORKDIR /usr/src/
RUN update-ca-certificates && \
    rustup target add x86_64-unknown-linux-gnu
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-gnu --release --package "wallet_provider" --bin wallet_provider

### Copy Shared Library Stage ###
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-hsm-pkcs11-library:latest AS shared_library

### Bundle Stage ###
FROM ${HARBOR_REGISTRY}/gcr-io-proxy/distroless/cc-debian12:nonroot

COPY --chown=nonroot --from=builder /usr/src/target/x86_64-unknown-linux-gnu/release/wallet_provider .

# relative to wallet_core context
COPY --chown=nonroot --from=shared_library /libcs_pkcs11_R3.so .

ENV WALLET_PROVIDER_HSM__LIBRARY_PATH=/home/nonroot/libcs_pkcs11_R3.so

CMD ["./wallet_provider"]
