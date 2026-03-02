import 'package:freezed_annotation/freezed_annotation.dart';

part 'flutter_api_error.freezed.dart';
part 'flutter_api_error.g.dart';

@Freezed(copyWith: false)
abstract class FlutterApiError with _$FlutterApiError {
  const factory FlutterApiError({
    required FlutterApiErrorType type,
    String? description,
    Map<String, dynamic>? data,
  }) = _FlutterApiError;

  factory FlutterApiError.fromJson(Map<String, dynamic> json) => _$FlutterApiErrorFromJson(json);
}

enum FlutterApiErrorType {
  @JsonValue('Generic')
  generic,
  @JsonValue('Server')
  server,
  @JsonValue('Networking')
  networking,
  @JsonValue('WalletState')
  walletState,
  @JsonValue('HardwareKeyUnsupported')
  hardwareKeyUnsupported,
  @JsonValue('RedirectUri')
  redirectUri,
  @JsonValue('DisclosureSourceMismatch')
  disclosureSourceMismatch,
  @JsonValue('ExpiredSession')
  expiredSession,
  @JsonValue('CancelledSession')
  cancelledSession,
  @JsonValue('Issuer')
  issuer,
  @JsonValue('Verifier')
  verifier,
  @JsonValue('WrongDigid')
  wrongDigid,
  @JsonValue('DeniedDigid')
  deniedDigid,
  @JsonValue('Revoked')
  revoked,
}
