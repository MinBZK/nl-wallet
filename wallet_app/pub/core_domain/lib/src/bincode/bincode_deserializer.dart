// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

part of bincode;

class BincodeDeserializer extends BinaryDeserializer {
  BincodeDeserializer(Uint8List input) : super(input);

  @override
  int deserializeLength() {
    // bincode sends this as a u64 but since transferred data length should never exceed the upper
    // bounds of an i64 (9223372036854775807 bytes is 9k petabytes) still deserialize to a Dart int
    return deserializeInt64();
  }

  @override
  int deserializeVariantIndex() {
    return deserializeUint32();
  }

  @override
  void checkThatKeySlicesAreIncreasing(Slice key1, Slice key2) {
    // Not required by the format.
  }
}
