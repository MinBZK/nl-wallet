class PinTimeoutScreenArgument {
  static const _kExpiryTimeKey = 'expiryTime';

  final DateTime expiryTime;

  const PinTimeoutScreenArgument({
    required this.expiryTime,
  });

  Map<String, dynamic> toMap() {
    return {
      _kExpiryTimeKey: expiryTime.toIso8601String(),
    };
  }

  static PinTimeoutScreenArgument fromMap(Map<String, dynamic> map) {
    return PinTimeoutScreenArgument(
      expiryTime: DateTime.parse(map[_kExpiryTimeKey]),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PinTimeoutScreenArgument && runtimeType == other.runtimeType && expiryTime == other.expiryTime;

  @override
  int get hashCode => expiryTime.hashCode;
}
