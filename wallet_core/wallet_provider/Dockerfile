### Copy Shared Library Stage ###
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-hsm-pkcs11-library:250122-13f7122c AS shared_library

### Bundle Stage ###
FROM ${HARBOR_REGISTRY}/gcr-io-proxy/distroless/cc-debian12:nonroot

# relative to wallet_core context
COPY --from=shared_library /libcs_pkcs11_R3.so .

COPY ./target/x86_64-unknown-linux-gnu/release/wallet_provider .

ENV WALLET_PROVIDER__HSM__LIBRARY_PATH=/home/nonroot/libcs_pkcs11_R3.so

CMD ["./wallet_provider"]
