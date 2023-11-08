class SignScreenArgument {
  static const _kSessionIdKey = 'sessionId';
  static const _kUriKey = 'uri';

  final String? mockSessionId;
  final String? uri;

  const SignScreenArgument({this.mockSessionId, this.uri}) : assert(mockSessionId != null || uri != null);

  Map<String, dynamic> toMap() {
    return {
      _kSessionIdKey: mockSessionId,
      _kUriKey: uri,
    };
  }

  static SignScreenArgument fromMap(Map<String, dynamic> map) {
    return SignScreenArgument(
      mockSessionId: map[_kSessionIdKey],
      uri: map[_kUriKey],
    );
  }

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
