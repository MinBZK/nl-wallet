FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/revocation_portal .
COPY revocation_portal/assets/ ./assets/

USER 1000
ENTRYPOINT ["./revocation_portal"]
