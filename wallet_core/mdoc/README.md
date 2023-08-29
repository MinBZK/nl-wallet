# nl-wallet-mdoc

A partial, work-in-progress Rust implementation of mdoc (ISO standards 18013-5, 23220-3, 23220-4).

## About mdoc

Studying mdoc and this repository can be done as follows.
- For an introduction to ISO mdoc and its most important data structures, see [`mdoc.md`](documentation/mdoc.md).
- View the documentation of this crate generated from the rustoc comments:
  ```sh
  cargo doc --open
  ```
- Some of the unit tests of this crate debug-print some mdoc data structures, which may be seen with `--nocapture`:
  ```sh
  cargo test -- iso_examples_disclosure --nocapture
  ```

## Running

Run the unit tests:

```sh
$ cargo test
```



## Organization

This crate is organized as follows.

- The ISO 18013 and 23220 data structures are implemented in the `iso` module, loosely grouped into separate modules under the `iso` folder.
  These modules contain almost exclusively data structures and no functions or methods, except for some conversion (`From`) methods;
  the functionality acting on these data structures is elsewhere in this crate.
  Encoding to the CBOR structure that the ISO standards mandate is done as much as possible by leveraging the type system and the default `serde` (de)serializers (augmented with `#[serde()]` attributes to get the right CBOR).
    - Most data types in the standard are defined as maps having fixed keys. These are implemented as Rust structs.
      However, for some data types, the standard demands that they should be serialized as a sequence (i.e. arrays, without field names) or as a map with incrementing integer values (also without field names).
      ISO 23220-3 additionally sometimes uses a map that has string keys which contain incrementing integer values ("0", "1", etc).
      In this case, we define an associated struct whose name ends on `Keyed` which does use field names.
      This allows us to refer to the contents of such data structures in the code by name instead of by numbers.
      During (de)serialization we transform them into the form required by the standard using the `CborSeq` and `CborIntMap` wrappers, which have custom (de)serializers for this.
    - Some CBOR data structures do not contain other data structures directly, but instead their CBOR-serialized bytes.
      For this the `TaggedBytes` wrapper is used.
    - Type aliases are used where possible, in particular when no methods need to be attached to the data structure.
- The three main agents (holder, issuer, verifier) each have their own module.
- The remaining functionality (serialization, cryptographic functions, and more) is placed in the `utils` module.

## Progress

- [x] Most of the datastructures in ISO 18013-5
- [x] Verification of mdocs and disclosures of mdocs
- [x] Issuance (ISO 23220-3 with an extension of the BasicSA application specific protocol)
- [ ] Close proximity transport channels: Bluetooth, NFC, Wifi Aware
- [ ] Online disclosure using ISO 23220-4
- [x] Support for external private keys (e.g. in mobile device secure hardware) using traits

The following functionality may or may be in scope for this crate, but is at any rate also not (yet) implemented:
- Issuer and verifier trusted registries
- Credential typing (i.e., defining the attribute structure of credentials)
