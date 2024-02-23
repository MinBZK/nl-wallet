import 'package:json_annotation/json_annotation.dart';

part 'edi_qr_code.g.dart';

@JsonSerializable()
class EdiQrCode {
  final String id;
  final EdiQrType type;

  const EdiQrCode({required this.id, required this.type});

  factory EdiQrCode.fromJson(Map<String, dynamic> json) {
    return _$EdiQrCodeFromJson(json);
  }

  Map<String, dynamic> toJson() => _$EdiQrCodeToJson(this);
}

enum EdiQrType {
  @JsonValue('issue') // Map old value 'issue' to 'issuance' type; for backwards QR code compatibility.
  issuance,
  @JsonValue('verify') // Map old value 'verify' to 'disclosure' type; for backwards QR code compatibility.
  disclosure,
  sign,
}
