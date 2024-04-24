ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:e8004706e1171f995fdfe38ed7c671b076eb4559e0cef3c1bc72690d19ab78b8 AS builder
WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "gba_hc_converter" --bin gba_hc_converter

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/gba_hc_converter .
COPY --from=builder /usr/src/gba_hc_converter/resources/stamdata /resources/stamdata
USER 1000
CMD ["./gba_hc_converter"]
