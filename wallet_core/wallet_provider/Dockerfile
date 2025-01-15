### Copy Shared Library Stage ###
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-hsm-pkcs11-library@sha256:2a4d7d5059fb8b7207cfa765966253af7b8d3031b0030037f68d3751f88325c5 AS shared_library

### Bundle Stage ###
FROM ${HARBOR_REGISTRY}/gcr-io-proxy/distroless/cc-debian12:nonroot

# relative to wallet_core context
COPY --from=shared_library /libcs_pkcs11_R3.so .

COPY ./target/x86_64-unknown-linux-gnu/release/wallet_provider .

ENV WALLET_PROVIDER_HSM__LIBRARY_PATH=/home/nonroot/libcs_pkcs11_R3.so

CMD ["./wallet_provider"]
