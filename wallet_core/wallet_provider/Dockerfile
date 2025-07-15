### Copy Shared Library Stage ###
FROM ${HARBOR_REGISTRY}/${HARBOR_PROJECT}/nl-wallet-hsm-pkcs11-client:${HSM_CLIENT_TAG} AS hsm-client

### Bundle Stage ###
FROM ${HARBOR_REGISTRY}/gcr-io-proxy/distroless/cc-debian12:nonroot

COPY --from=hsm-client /opt/utimaco/Crypto_APIs/PKCS11_R3/lib/libcs_pkcs11_R3.so .

COPY ./target/x86_64-unknown-linux-gnu/release/wallet_provider .

ENV WALLET_PROVIDER__HSM__LIBRARY_PATH=/home/nonroot/libcs_pkcs11_R3.so

CMD ["./wallet_provider"]
