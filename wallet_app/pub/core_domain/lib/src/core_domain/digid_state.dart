part of core_domain_types;

enum DigidState {
  authenticating,
  success,
  error,
}

extension DigidStateExtension on DigidState {
  static DigidState deserialize(BinaryDeserializer deserializer) {
    final index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return DigidState.authenticating;
      case 1: return DigidState.success;
      case 2: return DigidState.error;
      default: throw Exception("Unknown variant index for DigidState: " + index.toString());
    }
  }

  void serialize(BinarySerializer serializer) {
    switch (this) {
      case DigidState.authenticating: return serializer.serializeVariantIndex(0);
      case DigidState.success: return serializer.serializeVariantIndex(1);
      case DigidState.error: return serializer.serializeVariantIndex(2);
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static DigidState bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = DigidStateExtension.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}

