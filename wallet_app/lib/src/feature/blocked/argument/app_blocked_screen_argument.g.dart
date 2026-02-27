// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_blocked_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_AppBlockedScreenArgument _$AppBlockedScreenArgumentFromJson(
  Map<String, dynamic> json,
) => _AppBlockedScreenArgument(
  reason: $enumDecodeNullable(_$RevocationReasonEnumMap, json['reason']) ?? RevocationReason.unknown,
);

Map<String, dynamic> _$AppBlockedScreenArgumentToJson(
  _AppBlockedScreenArgument instance,
) => <String, dynamic>{'reason': _$RevocationReasonEnumMap[instance.reason]!};

const _$RevocationReasonEnumMap = {
  RevocationReason.adminRequest: 'admin_request',
  RevocationReason.userRequest: 'user_request',
  RevocationReason.unknown: 'unknown',
};
