FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/acf_demo_issuer_migrations .
USER 1000
ENTRYPOINT ["./acf_demo_issuer_migrations"]
CMD ["status"]
