import 'package:freezed_annotation/freezed_annotation.dart';

part 'revocation_data.freezed.dart';

part 'revocation_data.g.dart';

enum RevocationReason {
  @JsonValue('admin_request')
  adminRequest,
  @JsonValue('user_request')
  userRequest,
  @JsonValue('unknown')
  unknown,
}

@freezed
abstract class RevocationData with _$RevocationData {
  factory RevocationData({
    @JsonKey(name: 'revocation_reason', unknownEnumValue: RevocationReason.unknown)
    required RevocationReason revocationReason,
    @JsonKey(name: 'can_register_new_account') required bool canRegisterNewAccount,
  }) = _RevocationData;

  factory RevocationData.fromJson(Map<String, dynamic> json) => _$RevocationDataFromJson(json);

  factory RevocationData.unknown() => RevocationData(revocationReason: .unknown, canRegisterNewAccount: true);
}
