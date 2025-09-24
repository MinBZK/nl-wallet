import 'package:flutter/foundation.dart';

@immutable
class SignScreenArgument {
  static const _kSessionIdKey = 'sessionId';
  static const _kUriKey = 'uri';

  final String? mockSessionId;
  final String? uri;

  const SignScreenArgument({this.mockSessionId, this.uri})
    : assert(mockSessionId != null || uri != null, 'Either a mockSessionId of a uri is needed to start signing');

  Map<String, dynamic> toMap() {
    return {
      _kSessionIdKey: mockSessionId,
      _kUriKey: uri,
    };
  }

  SignScreenArgument.fromMap(Map<String, dynamic> map) : mockSessionId = map[_kSessionIdKey], uri = map[_kUriKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SignScreenArgument &&
          runtimeType == other.runtimeType &&
          mockSessionId == other.mockSessionId &&
          uri == other.uri;

  @override
  int get hashCode => Object.hash(
    runtimeType,
    mockSessionId,
    uri,
  );
}
