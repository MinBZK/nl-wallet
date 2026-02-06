FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/audit_log_migrations .
USER 1000
ENTRYPOINT ["./audit_log_migrations"]
CMD ["status"]
