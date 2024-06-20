ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:a6f0c5662e128280e63637e3d2eb2620b260ef921eda9d1946546098a7c22446 AS builder
WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "pid_issuer" --bin pid_issuer

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/pid_issuer .
COPY --from=builder /etc/ssl/certs/ /etc/ssl/certs/
USER 1000
CMD ["./pid_issuer"]
