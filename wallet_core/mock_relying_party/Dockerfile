ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:a6f0c5662e128280e63637e3d2eb2620b260ef921eda9d1946546098a7c22446 AS builder
WORKDIR /usr/src/
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "mock_relying_party" --bin mock_relying_party

# Bundle Stage
FROM scratch

COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/mock_relying_party .
COPY --from=builder /etc/ssl/certs/ /etc/ssl/certs/
COPY mock_relying_party/assets/ ./assets/

ENTRYPOINT ["./mock_relying_party"]
