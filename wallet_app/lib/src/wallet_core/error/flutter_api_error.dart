import 'package:json_annotation/json_annotation.dart';

part 'flutter_api_error.g.dart';

@JsonSerializable()
class FlutterApiError {
  FlutterApiErrorType type;
  String? description;
  Map<String, dynamic>? data;

  FlutterApiError({required this.type, this.description, this.data});

  factory FlutterApiError.fromJson(Map<String, dynamic> json) => _$FlutterApiErrorFromJson(json);

  Map<String, dynamic> toJson() => _$FlutterApiErrorToJson(this);

  @override
  String toString() => 'FlutterApiError{type: ${type.name}, description: $description}';
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
}
