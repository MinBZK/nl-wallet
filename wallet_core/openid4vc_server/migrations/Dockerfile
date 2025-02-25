FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/openid4vc_server_migrations .
USER 1000
ENTRYPOINT ["./openid4vc_server_migrations"]
CMD ["status"]
