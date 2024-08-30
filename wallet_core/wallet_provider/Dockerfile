### Copy Shared Library Stage ###
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-hsm-pkcs11-library:latest AS shared_library

### Bundle Stage ###
FROM ${HARBOR_REGISTRY}/gcr-io-proxy/distroless/cc-debian12:nonroot

COPY --chown=nonroot ./target/x86_64-unknown-linux-gnu/release/wallet_provider .

# relative to wallet_core context
COPY --chown=nonroot --from=shared_library /libcs_pkcs11_R3.so .

ENV WALLET_PROVIDER_HSM__LIBRARY_PATH=/home/nonroot/libcs_pkcs11_R3.so

CMD ["./wallet_provider"]
