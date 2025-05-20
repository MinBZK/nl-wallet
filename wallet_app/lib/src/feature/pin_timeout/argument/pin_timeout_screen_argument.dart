import 'package:equatable/equatable.dart';
import 'package:flutter/foundation.dart';

@immutable
class PinTimeoutScreenArgument extends Equatable {
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
  List<Object?> get props => [expiryTime];
}
