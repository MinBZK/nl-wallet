ARG HARBOR_REGISTRY
ARG HARBOR_NLW_PROJECT

# Build Stage
FROM ${HARBOR_REGISTRY}/${HARBOR_NLW_PROJECT}/nl-wallet-app-builder-rust@sha256:befd6633cdf0ea52c4e99ae08dd4746eed327e4467d2236df5ddff1f8eb9fc6a AS builder
WORKDIR /usr/src/
COPY . .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release --package "mock_relying_party" --bin mock_relying_party

# Bundle Stage
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/mock_relying_party .
COPY --from=builder /etc/ssl/certs/ /etc/ssl/certs/
USER 1000
CMD ["./mock_relying_party"]
