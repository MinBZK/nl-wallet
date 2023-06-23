class IssuanceScreenArgument {
  static const _kSessionIdKey = 'sessionId';
  static const _kIsRefreshFlow = 'isRefreshFlow';

  final String sessionId;
  final bool isRefreshFlow;

  const IssuanceScreenArgument({required this.sessionId, this.isRefreshFlow = false});

  Map<String, dynamic> toMap() {
    return {
      _kSessionIdKey: sessionId,
      _kIsRefreshFlow: isRefreshFlow,
    };
  }

  static IssuanceScreenArgument fromMap(Map<String, dynamic> map) {
    return IssuanceScreenArgument(
      sessionId: map[_kSessionIdKey],
      isRefreshFlow: map[_kIsRefreshFlow],
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is IssuanceScreenArgument &&
          runtimeType == other.runtimeType &&
          sessionId == other.sessionId &&
          isRefreshFlow == other.isRefreshFlow;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        sessionId,
        isRefreshFlow,
      );
}
