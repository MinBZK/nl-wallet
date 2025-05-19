FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/gba_fetch_frontend .
USER 1000
CMD ["./gba_fetch_frontend"]
