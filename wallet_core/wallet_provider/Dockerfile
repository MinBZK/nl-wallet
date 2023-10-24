ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

### Build Stage ###
# rust 1.73
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:3ce69596778f722dae23f64f40240c490f4686d5b361f116d98b507057768b8b AS builder
WORKDIR /usr/src/
RUN update-ca-certificates && \
    rustup target add x86_64-unknown-linux-gnu
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-gnu --release --package "wallet_provider" --bin wallet_provider

### Copy Shared Library Stage ###
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-hsm-pkcs11-library:latest AS shared_library

### Bundle Stage ###
FROM ${HARBOR_REGISTRY}/gcr-io-proxy/distroless/cc-debian12

COPY --from=builder /usr/src/target/x86_64-unknown-linux-gnu/release/wallet_provider .

# relative to wallet_core context
COPY --from=shared_library /libcs_pkcs11_R3.so .

ENV WALLET_PROVIDER_HSM__LIBRARY_PATH=/libcs_pkcs11_R3.so

USER 1000
CMD ["./wallet_provider"]
