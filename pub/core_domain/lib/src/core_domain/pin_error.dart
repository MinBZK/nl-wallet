part of core_domain_types;

enum PinError {
  nonDigits,
  invalidLength,
  tooFewUniqueDigits,
  ascendingDigits,
  descendingDigits,
}

extension PinErrorExtension on PinError {
  static PinError deserialize(BinaryDeserializer deserializer) {
    final index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return PinError.nonDigits;
      case 1: return PinError.invalidLength;
      case 2: return PinError.tooFewUniqueDigits;
      case 3: return PinError.ascendingDigits;
      case 4: return PinError.descendingDigits;
      default: throw Exception("Unknown variant index for PinError: " + index.toString());
    }
  }

  void serialize(BinarySerializer serializer) {
    switch (this) {
      case PinError.nonDigits: return serializer.serializeVariantIndex(0);
      case PinError.invalidLength: return serializer.serializeVariantIndex(1);
      case PinError.tooFewUniqueDigits: return serializer.serializeVariantIndex(2);
      case PinError.ascendingDigits: return serializer.serializeVariantIndex(3);
      case PinError.descendingDigits: return serializer.serializeVariantIndex(4);
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static PinError bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = PinErrorExtension.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}

