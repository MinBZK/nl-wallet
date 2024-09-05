FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/wallet_provider_migrations .
USER 1000
ENTRYPOINT ["./wallet_provider_migrations"]
CMD ["status"]
