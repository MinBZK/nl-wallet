ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:4a4f49ce40693fbc9040869aa98822de3bfa87594ba7c7f25e5091f21a943fe5 AS builder
WORKDIR /usr/src/
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "gba_hc_converter" --bin gba_hc_converter

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/gba_hc_converter .
COPY --from=builder /usr/src/gba_hc_converter/resources/remote/bsn_zoeken_template.xml /resources/remote/bsn_zoeken_template.xml
COPY --from=builder /usr/src/gba_hc_converter/resources/stamdata /resources/stamdata
USER 1000
CMD ["./gba_hc_converter"]
