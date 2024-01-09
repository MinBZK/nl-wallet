ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
# rust 1.71
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:201d58b0c6ed13531383011a0bed692f303dc3ec345ff139f72c17894ad26a63 AS builder
WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "mock_relying_party" --bin mock_relying_party

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/mock_relying_party .
COPY --from=builder /etc/ssl/certs/ /etc/ssl/certs/
USER 1000
CMD ["./mock_relying_party"]
