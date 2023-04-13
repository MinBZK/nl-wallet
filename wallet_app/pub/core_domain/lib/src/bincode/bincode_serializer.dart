// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

part of bincode;

class BincodeSerializer extends BinarySerializer {
  @override
  void serializeLength(int value) {
    // bincode expects a u64 but since the capacity of a Dart int is less than that
    // we can safely serialize as int to simplify over constructing Uint8 bytes
    return serializeInt64(value);
  }

  @override
  void serializeVariantIndex(int value) {
    serializeUint32(value);
  }

  void sortMapEntries(Int32List offsets) {
    // Not required by the format.
  }
}
