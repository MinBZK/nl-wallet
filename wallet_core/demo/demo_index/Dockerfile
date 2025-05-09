FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/demo_index .
COPY demo/demo_index/assets/ ./assets/
COPY demo/demo_utils/assets/ ./assets/

USER 1000
ENTRYPOINT ["./demo_index"]
