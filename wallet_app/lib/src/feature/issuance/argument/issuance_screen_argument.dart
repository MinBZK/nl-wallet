import 'package:flutter/foundation.dart';

@immutable
class IssuanceScreenArgument {
  static const _kSessionIdKey = 'sessionId';
  static const _kIsRefreshFlowKey = 'isRefreshFlow';
  static const _kUriKey = 'uri';

  final String? mockSessionId;
  final bool isRefreshFlow;
  final String? uri;

  const IssuanceScreenArgument({this.mockSessionId, this.isRefreshFlow = false, this.uri})
      : assert(mockSessionId != null || uri != null, 'Either a mockSessionId of a uri is needed to start issuance');

  Map<String, dynamic> toMap() {
    return {
      _kSessionIdKey: mockSessionId,
      _kIsRefreshFlowKey: isRefreshFlow,
      _kUriKey: uri,
    };
  }

  IssuanceScreenArgument.fromMap(Map<String, dynamic> map)
      : mockSessionId = map[_kSessionIdKey],
        isRefreshFlow = map[_kIsRefreshFlowKey],
        uri = map[_kUriKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is IssuanceScreenArgument &&
          runtimeType == other.runtimeType &&
          mockSessionId == other.mockSessionId &&
          isRefreshFlow == other.isRefreshFlow;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        mockSessionId,
        isRefreshFlow,
      );
}
