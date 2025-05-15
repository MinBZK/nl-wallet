import 'package:equatable/equatable.dart';
import 'package:flutter/foundation.dart';

@immutable
class IssuanceScreenArgument extends Equatable {
  static const _kSessionIdKey = 'sessionId';
  static const _kIsRefreshFlowKey = 'isRefreshFlow';
  static const _kUriKey = 'uri';
  static const _kIsQrCodeKey = 'qrCode';

  final String? mockSessionId;
  final bool isQrCode;
  final bool isRefreshFlow;
  final String? uri;

  const IssuanceScreenArgument({this.mockSessionId, required this.isQrCode, this.isRefreshFlow = false, this.uri})
      : assert(mockSessionId != null || uri != null, 'Either a mockSessionId of a uri is needed to start issuance');

  Map<String, dynamic> toMap() {
    return {
      _kSessionIdKey: mockSessionId,
      _kIsRefreshFlowKey: isRefreshFlow,
      _kUriKey: uri,
      _kIsQrCodeKey: isQrCode,
    };
  }

  IssuanceScreenArgument.fromMap(Map<String, dynamic> map)
      : mockSessionId = map[_kSessionIdKey],
        isRefreshFlow = map[_kIsRefreshFlowKey],
        isQrCode = map[_kIsQrCodeKey],
        uri = map[_kUriKey];

  @override
  List<Object?> get props => [uri, isQrCode, isRefreshFlow, uri];
}
