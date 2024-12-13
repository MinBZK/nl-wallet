FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/update_policy_server .

USER 1000
ENTRYPOINT ["./update_policy_server"]
