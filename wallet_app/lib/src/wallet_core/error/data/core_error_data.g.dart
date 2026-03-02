// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'core_error_data.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_CoreErrorData _$CoreErrorDataFromJson(Map<String, dynamic> json) => _CoreErrorData(
  revocationData: json['revocation_data'] == null
      ? null
      : RevocationData.fromJson(
          json['revocation_data'] as Map<String, dynamic>,
        ),
  redirectError: $enumDecodeNullable(
    _$RedirectErrorEnumMap,
    json['redirect_error'],
    unknownValue: RedirectError.unknown,
  ),
  sessionType: $enumDecodeNullable(
    _$SessionTypeEnumMap,
    json['session_type'],
    unknownValue: SessionType.unknown,
  ),
  canRetry: json['can_retry'] as bool?,
  organizationName: json['organization_name'] as Map<String, dynamic>?,
);

Map<String, dynamic> _$CoreErrorDataToJson(_CoreErrorData instance) => <String, dynamic>{
  'revocation_data': instance.revocationData?.toJson(),
  'redirect_error': _$RedirectErrorEnumMap[instance.redirectError],
  'session_type': _$SessionTypeEnumMap[instance.sessionType],
  'can_retry': instance.canRetry,
  'organization_name': instance.organizationName,
};

const _$RedirectErrorEnumMap = {
  RedirectError.accessDenied: 'access_denied',
  RedirectError.serverError: 'server_error',
  RedirectError.loginRequired: 'login_required',
  RedirectError.unknown: 'unknown',
};

const _$SessionTypeEnumMap = {
  SessionType.sameDevice: 'same_device',
  SessionType.crossDevice: 'cross_device',
  SessionType.unknown: 'unknown',
};
