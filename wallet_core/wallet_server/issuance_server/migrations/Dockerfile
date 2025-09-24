FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/issuance_server_migrations .
USER 1000
ENTRYPOINT ["./issuance_server_migrations"]
CMD ["status"]
