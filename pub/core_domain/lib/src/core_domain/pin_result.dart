part of core_domain_types;

enum PinResult {
  ok,
  tooFewUniqueDigitsError,
  sequentialDigitsError,
  otherError,
}

extension PinResultExtension on PinResult {
  static PinResult deserialize(BinaryDeserializer deserializer) {
    final index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return PinResult.ok;
      case 1: return PinResult.tooFewUniqueDigitsError;
      case 2: return PinResult.sequentialDigitsError;
      case 3: return PinResult.otherError;
      default: throw Exception("Unknown variant index for PinResult: " + index.toString());
    }
  }

  void serialize(BinarySerializer serializer) {
    switch (this) {
      case PinResult.ok: return serializer.serializeVariantIndex(0);
      case PinResult.tooFewUniqueDigitsError: return serializer.serializeVariantIndex(1);
      case PinResult.sequentialDigitsError: return serializer.serializeVariantIndex(2);
      case PinResult.otherError: return serializer.serializeVariantIndex(3);
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static PinResult bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = PinResultExtension.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}

