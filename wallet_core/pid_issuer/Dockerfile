ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
# rust 1.71
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:d308bc4581520b481dd6f78dd9da205fc98d99ce27e1a9069dca05b3ee74e404 AS builder
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
