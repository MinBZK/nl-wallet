part of core_domain_types;

abstract class WalletUnlockResult {
  const WalletUnlockResult();

  void serialize(BinarySerializer serializer);

  static WalletUnlockResult deserialize(BinaryDeserializer deserializer) {
    int index = deserializer.deserializeVariantIndex();
    switch (index) {
      case 0: return WalletUnlockResultOk.load(deserializer);
      case 1: return WalletUnlockResultIncorrectPin.load(deserializer);
      case 2: return WalletUnlockResultTimeout.load(deserializer);
      case 3: return WalletUnlockResultBlocked.load(deserializer);
      case 4: return WalletUnlockResultServerError.load(deserializer);
      default: throw Exception("Unknown variant index for WalletUnlockResult: " + index.toString());
    }
  }

  Uint8List bincodeSerialize() {
      final serializer = BincodeSerializer();
      serialize(serializer);
      return serializer.bytes;
  }

  static WalletUnlockResult bincodeDeserialize(Uint8List input) {
    final deserializer = BincodeDeserializer(input);
    final value = WalletUnlockResult.deserialize(deserializer);
    if (deserializer.offset < input.length) {
      throw Exception('Some input bytes were not read');
    }
    return value;
  }
}


@immutable
class WalletUnlockResultOk extends WalletUnlockResult {
  const WalletUnlockResultOk(
  ) : super();

  WalletUnlockResultOk.load(BinaryDeserializer deserializer);

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(0);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is WalletUnlockResultOk;
  }

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        ')';
      return true;
    }());

    return fullString ?? 'WalletUnlockResultOk';
  }
}

@immutable
class WalletUnlockResultIncorrectPin extends WalletUnlockResult {
  const WalletUnlockResultIncorrectPin({
    required this.leftoverAttempts,
    required this.isFinalAttempt,
  }) : super();

  WalletUnlockResultIncorrectPin.load(BinaryDeserializer deserializer) :
    leftoverAttempts = deserializer.deserializeUint8(),
    isFinalAttempt = deserializer.deserializeBool();

  final int leftoverAttempts;
  final bool isFinalAttempt;

  WalletUnlockResultIncorrectPin copyWith({
    int? leftoverAttempts,
    bool? isFinalAttempt,
  }) {
    return WalletUnlockResultIncorrectPin(
      leftoverAttempts: leftoverAttempts ?? this.leftoverAttempts,
      isFinalAttempt: isFinalAttempt ?? this.isFinalAttempt,
    );
  }

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(1);
    serializer.serializeUint8(leftoverAttempts);
    serializer.serializeBool(isFinalAttempt);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is WalletUnlockResultIncorrectPin
      && leftoverAttempts == other.leftoverAttempts
      && isFinalAttempt == other.isFinalAttempt;
  }

  @override
  int get hashCode => Object.hash(
        leftoverAttempts,
        isFinalAttempt,
      );

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        'leftoverAttempts: $leftoverAttempts, '
        'isFinalAttempt: $isFinalAttempt'
        ')';
      return true;
    }());

    return fullString ?? 'WalletUnlockResultIncorrectPin';
  }
}

@immutable
class WalletUnlockResultTimeout extends WalletUnlockResult {
  const WalletUnlockResultTimeout({
    required this.timeoutMillis,
  }) : super();

  WalletUnlockResultTimeout.load(BinaryDeserializer deserializer) :
    timeoutMillis = deserializer.deserializeUint32();

  final int timeoutMillis;

  WalletUnlockResultTimeout copyWith({
    int? timeoutMillis,
  }) {
    return WalletUnlockResultTimeout(
      timeoutMillis: timeoutMillis ?? this.timeoutMillis,
    );
  }

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(2);
    serializer.serializeUint32(timeoutMillis);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is WalletUnlockResultTimeout
      && timeoutMillis == other.timeoutMillis;
  }

  @override
  int get hashCode => timeoutMillis.hashCode;

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        'timeoutMillis: $timeoutMillis'
        ')';
      return true;
    }());

    return fullString ?? 'WalletUnlockResultTimeout';
  }
}

@immutable
class WalletUnlockResultBlocked extends WalletUnlockResult {
  const WalletUnlockResultBlocked(
  ) : super();

  WalletUnlockResultBlocked.load(BinaryDeserializer deserializer);

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(3);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is WalletUnlockResultBlocked;
  }

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        ')';
      return true;
    }());

    return fullString ?? 'WalletUnlockResultBlocked';
  }
}

@immutable
class WalletUnlockResultServerError extends WalletUnlockResult {
  const WalletUnlockResultServerError(
  ) : super();

  WalletUnlockResultServerError.load(BinaryDeserializer deserializer);

  void serialize(BinarySerializer serializer) {
    serializer.serializeVariantIndex(4);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is WalletUnlockResultServerError;
  }

  @override
  String toString() {
    String? fullString;

    assert(() {
      fullString = '$runtimeType('
        ')';
      return true;
    }());

    return fullString ?? 'WalletUnlockResultServerError';
  }
}
