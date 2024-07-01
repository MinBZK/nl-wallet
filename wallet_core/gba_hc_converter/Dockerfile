ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:a6f0c5662e128280e63637e3d2eb2620b260ef921eda9d1946546098a7c22446 AS builder
WORKDIR /usr/src/
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "gba_hc_converter" --bin gba_hc_converter

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/gba_hc_converter .
COPY --from=builder /usr/src/gba_hc_converter/resources/remote/bsn_zoeken_template.xml .
COPY --from=builder /usr/src/gba_hc_converter/resources/stamdata /resources/stamdata
USER 1000
CMD ["./gba_hc_converter"]
