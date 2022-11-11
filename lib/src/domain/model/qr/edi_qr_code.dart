import 'package:json_annotation/json_annotation.dart';

part 'edi_qr_code.g.dart';

@JsonSerializable()
class EdiQrCode {
  final int id;
  final EdiQrType type;

  EdiQrCode({required this.id, required this.type});

  factory EdiQrCode.fromJson(Map<String, dynamic> json) {
    return _$EdiQrCodeFromJson(json);
  }

  Map<String, dynamic> toJson() => _$EdiQrCodeToJson(this);
}

enum EdiQrType { issue, verify }
