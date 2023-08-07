# PID Issuer

This folder contains a reference implementation for a PID issuer.

At the moment this application is set up for connecting to the digid-connector.

# Development environment

In order to test this issuer locally, and connect the wallet to it, follow the following steps:

1. Generate a 2048-bit RSA key pair and provision the digid-connect with the public key.
2. Place the private key in JWK format a file named `private_key.jwk`, the place that file in a subdirectory of this file called `secrets`.
3. Run the pid_issuer with `RUST_LOG=DEBUG cargo run --bin pid_issuer`.
4. Modify the `PID_ISSUER_BASE_URL` variable in `digid.rs` to: "http://10.0.2.2:3003/" (This works for the android emulator, it should be "http://localhost:3003/" for the iOS simulator).
5. Start the wallet.
