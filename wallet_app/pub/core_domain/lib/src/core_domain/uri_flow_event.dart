part of core_domain_types;

abstract class UriFlowEvent {
  const UriFlowEvent();

  void serialize(BinarySerializer serializer);

  static UriFlowEvent deserialize(BinaryDeserializer deserializer) {
    int index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return UriFlowEventDigidAuth.load(deserializer);
      default: throw Exception("Unknown variant index for UriFlowEvent: " + index.toString());
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static UriFlowEvent bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = UriFlowEvent.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}


@immutable
class UriFlowEventDigidAuth extends UriFlowEvent {
  const UriFlowEventDigidAuth({
    required this.state,
  }) : super();

  UriFlowEventDigidAuth.load(BinaryDeserializer deserializer) :
    state = DigidStateExtension.deserialize(deserializer);

  final DigidState state;

  UriFlowEventDigidAuth copyWith({
    DigidState? state,
  }) {
    return UriFlowEventDigidAuth(
      state: state ?? this.state,
    );
  }

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(0);
    state.serialize(serializer);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is UriFlowEventDigidAuth
      && state == other.state;
  }

  @override
  int get hashCode => state.hashCode;

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        'state: $state'
        ')';
      return true;
    }());

    return fullString ?? 'UriFlowEventDigidAuth';
  }
}
