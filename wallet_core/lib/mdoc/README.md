# nl-wallet-mdoc

A partial, work-in-progress Rust implementation of mdoc (ISO standard 18013-5).

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

- The ISO 18013 data structures are implemented in the `iso` module, loosely grouped into separate modules under the `iso` folder.
  These modules contain almost exclusively data structures and no functions or methods, except for some conversion (`From`) methods;
  the functionality acting on these data structures is elsewhere in this crate.
  Encoding to the CBOR structure that the ISO standards mandate is done as much as possible by leveraging the type system and the default `serde` (de)serializers (augmented with `#[serde()]` attributes to get the right CBOR).
    - Most data types in the standard are defined as maps having fixed keys. These are implemented as Rust structs.
      However, for some data types, the standard demands that they should be serialized as a sequence (i.e. arrays, without field names) or as a map with incrementing integer values (also without field names).
      In this case, we define an associated struct whose name ends on `Keyed` which does use field names.
      This allows us to refer to the contents of such data structures in the code by name instead of by numbers.
      During (de)serialization we transform them into the form required by the standard using the `CborSeq` and `CborIntMap` wrappers, which have custom (de)serializers for this.
    - Some CBOR data structures do not contain other data structures directly, but instead their CBOR-serialized bytes.
      For this the `TaggedBytes` wrapper is used.
    - Type aliases are used where possible, in particular when no methods need to be attached to the data structure.
- The remaining functionality (serialization, cryptographic functions, and more) is placed in the `utils` module.

## Progress

- [x] Most of the datastructures in ISO 18013-5
- [x] Utility code to verify of mdocs and aid disclosure of mdocs
- [ ] Close proximity transport channels: Bluetooth, NFC, Wifi Aware
- [x] Support for external private keys (e.g. in mobile device secure hardware) using traits

The following functionality may or may be in scope for this crate, but is at any rate also not (yet) implemented:
- Issuer and verifier trusted registries
- Credential typing (i.e., defining the attribute structure of credentials)
