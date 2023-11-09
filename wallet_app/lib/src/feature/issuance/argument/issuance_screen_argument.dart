class IssuanceScreenArgument {
  static const _kSessionIdKey = 'sessionId';
  static const _kIsRefreshFlowKey = 'isRefreshFlow';
  static const _kUriKey = 'uri';

  final String? mockSessionId;
  final bool isRefreshFlow;
  final String? uri;

  const IssuanceScreenArgument({this.mockSessionId, this.isRefreshFlow = false, this.uri})
      : assert(mockSessionId != null || uri != null);

  Map<String, dynamic> toMap() {
    return {
      _kSessionIdKey: mockSessionId,
      _kIsRefreshFlowKey: isRefreshFlow,
      _kUriKey: uri,
    };
  }

  static IssuanceScreenArgument fromMap(Map<String, dynamic> map) {
    return IssuanceScreenArgument(
      mockSessionId: map[_kSessionIdKey],
      isRefreshFlow: map[_kIsRefreshFlowKey],
      uri: map[_kUriKey],
    );
  }

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
