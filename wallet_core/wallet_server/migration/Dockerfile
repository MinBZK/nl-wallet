FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/wallet_server_migration .
USER 1000
ENTRYPOINT ["./wallet_server_migration"]
CMD ["status"]
