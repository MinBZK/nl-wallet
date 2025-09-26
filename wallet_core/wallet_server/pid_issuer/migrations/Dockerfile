FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/pid_issuer_migrations .
USER 1000
ENTRYPOINT ["./pid_issuer_migrations"]
CMD ["status"]
