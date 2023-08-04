# PID Issuer

This folder contains a reference implementation for a PID issuer.

At the moment this application connects to the digid-connector running on example.com.

# Development environment

In order to test this issuer locally, and connect the wallet to it, follow the following steps:

1. Download the `nl-pid-issuer-secrets` secret
2. Extract the `rsa_private.jwk` from the secret, and store in a folder called `secrets`
3. Run the pid_issuer with `RUST_LOG=DEBUG cargo run --bin pid_issuer`
4. Modify the `PID_ISSUER_BASE_URL` variable in `digid.rs` to: "http://10.0.2.2:3003/" (This works for the android emulator, it probably works with "http://localhost:3003/" on iOS.)
5. Start the wallet
