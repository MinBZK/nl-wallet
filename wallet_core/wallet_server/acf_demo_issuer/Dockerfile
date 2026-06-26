FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/acf_demo_issuer .
COPY wallet_server/acf_demo_issuer/assets/ ./assets/

USER 1000
CMD ["./acf_demo_issuer"]
