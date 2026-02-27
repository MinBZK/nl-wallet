// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'revocation_data.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_RevocationData _$RevocationDataFromJson(Map<String, dynamic> json) => _RevocationData(
  revocationReason: $enumDecode(
    _$RevocationReasonEnumMap,
    json['revocation_reason'],
    unknownValue: RevocationReason.unknown,
  ),
  canRegisterNewAccount: json['can_register_new_account'] as bool,
);

Map<String, dynamic> _$RevocationDataToJson(
  _RevocationData instance,
) => <String, dynamic>{
  'revocation_reason': _$RevocationReasonEnumMap[instance.revocationReason]!,
  'can_register_new_account': instance.canRegisterNewAccount,
};

const _$RevocationReasonEnumMap = {
  RevocationReason.adminRequest: 'admin_request',
  RevocationReason.userRequest: 'user_request',
  RevocationReason.unknown: 'unknown',
};
