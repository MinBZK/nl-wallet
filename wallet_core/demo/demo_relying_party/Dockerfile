FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/demo_relying_party .
COPY demo/demo_relying_party/assets/ ./assets/
COPY demo/demo_utils/assets/ ./assets/

USER 1000
ENTRYPOINT ["./demo_relying_party"]
