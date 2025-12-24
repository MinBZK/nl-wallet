ARG HARBOR_REGISTRY
ARG GCR_IO_PROXY=gcr.io
FROM ${GCR_IO_PROXY}/distroless/cc-debian12:debug-nonroot

COPY ./target/x86_64-unknown-linux-musl/debug/gba_encrypt .
COPY ./target/x86_64-unknown-linux-musl/debug/gba_fetch .

COPY --chown=nonroot ./gba_hc_converter/resources/gba-v-responses ./unencrypted-gba-v-responses

ENTRYPOINT ["sh"]
