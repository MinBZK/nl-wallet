part of core_domain_types;

abstract class PinResult {
  const PinResult();

  void serialize(BinarySerializer serializer);

  static PinResult deserialize(BinaryDeserializer deserializer) {
    int index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return PinResultOkItem.load(deserializer);
      case 1: return PinResultErrItem.load(deserializer);
      default: throw Exception("Unknown variant index for PinResult: " + index.toString());
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static PinResult bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = PinResult.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}


@immutable
class PinResultOkItem extends PinResult {
  const PinResultOkItem(
  ) : super();

  PinResultOkItem.load(BinaryDeserializer deserializer);

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(0);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is PinResultOkItem
    ;}

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        ')';
      return true;
    }());

    return fullString ?? 'PinResultOkItem';
  }
}

@immutable
class PinResultErrItem extends PinResult {
  const PinResultErrItem({
    required this.value,
  }) : super();

  PinResultErrItem.load(BinaryDeserializer deserializer) :
    value = PinErrorExtension.deserialize(deserializer);

  final PinError value;


  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(1);
    value.serialize(serializer);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is PinResultErrItem
    &&  value == other.value
    ;}

  @override
  int get hashCode => value.hashCode;

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        'value: $value'
        ')';
      return true;
    }());

    return fullString ?? 'PinResultErrItem';
  }
}
