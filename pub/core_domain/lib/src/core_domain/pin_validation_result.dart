part of core_domain_types;

enum PinValidationResult {
  ok,
  nonDigitsError,
  invalidLengthError,
  tooFewUniqueDigitsError,
  ascendingDigitsError,
  descendingDigitsError,
}

extension PinValidationResultExtension on PinValidationResult {
  static PinValidationResult deserialize(BinaryDeserializer deserializer) {
    final index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return PinValidationResult.ok;
      case 1: return PinValidationResult.nonDigitsError;
      case 2: return PinValidationResult.invalidLengthError;
      case 3: return PinValidationResult.tooFewUniqueDigitsError;
      case 4: return PinValidationResult.ascendingDigitsError;
      case 5: return PinValidationResult.descendingDigitsError;
      default: throw Exception("Unknown variant index for PinValidationResult: " + index.toString());
    }
  }

  void serialize(BinarySerializer serializer) {
    switch (this) {
      case PinValidationResult.ok: return serializer.serializeVariantIndex(0);
      case PinValidationResult.nonDigitsError: return serializer.serializeVariantIndex(1);
      case PinValidationResult.invalidLengthError: return serializer.serializeVariantIndex(2);
      case PinValidationResult.tooFewUniqueDigitsError: return serializer.serializeVariantIndex(3);
      case PinValidationResult.ascendingDigitsError: return serializer.serializeVariantIndex(4);
      case PinValidationResult.descendingDigitsError: return serializer.serializeVariantIndex(5);
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static PinValidationResult bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = PinValidationResultExtension.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}

