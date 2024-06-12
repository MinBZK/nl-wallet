import 'package:flutter/foundation.dart';

@immutable
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

  PinTimeoutScreenArgument.fromMap(Map<String, dynamic> map) : expiryTime = DateTime.parse(map[_kExpiryTimeKey]);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PinTimeoutScreenArgument && runtimeType == other.runtimeType && expiryTime == other.expiryTime;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        expiryTime,
      );
}
