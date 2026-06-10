FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/pacf_issuance_server_migrations .
USER 1000
ENTRYPOINT ["./pacf_issuance_server_migrations"]
CMD ["status"]
