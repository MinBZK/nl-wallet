import 'package:freezed_annotation/freezed_annotation.dart';

part 'edi_qr_code.freezed.dart';
part 'edi_qr_code.g.dart';

@Freezed(copyWith: false)
abstract class EdiQrCode with _$EdiQrCode {
  const factory EdiQrCode({
    required String id,
    required EdiQrType type,
  }) = _EdiQrCode;

  factory EdiQrCode.fromJson(Map<String, dynamic> json) => _$EdiQrCodeFromJson(json);
}

enum EdiQrType {
  @JsonValue('issue') // Map old value 'issue' to 'issuance' type; for backwards QR code compatibility.
  issuance,
  @JsonValue('verify') // Map old value 'verify' to 'disclosure' type; for backwards QR code compatibility.
  disclosure,
  sign,
}
