FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/wallet_server_migrations .
USER 1000
ENTRYPOINT ["./wallet_server_migrations"]
CMD ["status"]
