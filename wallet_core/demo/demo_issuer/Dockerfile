FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/demo_issuer .
COPY demo/demo_issuer/assets/ ./assets/

USER 1000
ENTRYPOINT ["./demo_issuer"]
