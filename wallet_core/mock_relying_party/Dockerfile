ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:4a4f49ce40693fbc9040869aa98822de3bfa87594ba7c7f25e5091f21a943fe5 AS builder
WORKDIR /usr/src/
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "mock_relying_party" --bin mock_relying_party

# Bundle Stage
FROM scratch

COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/mock_relying_party .
COPY --from=builder /etc/ssl/certs/ /etc/ssl/certs/
COPY mock_relying_party/assets/ ./assets/

ENTRYPOINT ["./mock_relying_party"]
