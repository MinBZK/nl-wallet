FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/verification_server_migrations .
USER 1000
ENTRYPOINT ["./verification_server_migrations"]
CMD ["status"]
