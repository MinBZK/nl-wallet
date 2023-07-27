# mdoc

A partial, work-in-progress Rust implementation of mdoc (ISO standards 18013-5, 23220-3, 23220-4).

## Running

Run its unit tests:

```sh
$ cargo test
```

Some of these debug-print some mdoc data structures, which may be seen with `--nocapture`:

```sh
$ cargo test -- iso_examples_disclosure --nocapture
```

## Progress

- [x] Most of the datastructures in ISO 18013-5
- [x] Verification of mdocs and disclosures of mdocs
- [x] Issuance (using an ad-hoc protocol instead of ISO 23220-3)
- [ ] Close proximity transport channels: Bluetooth, NFC, Wifi Aware
- [ ] Issuance using ISO 23220-3
- [ ] Online disclosure using ISO 23220-4
- [ ] Support for credential private keys in the mobile device's secure hardware

The following functionality may or may be in scope for this crate, but is at any rate also not (yet) implemented:
- Session bookkeeping for the holder, verifier, and issuer
- Issuer and verifier trusted registries
- Storing and loading to disk of credentials, keys, or anything else
- Credential typing (i.e., defining the attribute structure of credentials)
