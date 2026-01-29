// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'issuance_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_IssuanceScreenArgument _$IssuanceScreenArgumentFromJson(
  Map<String, dynamic> json,
) => _IssuanceScreenArgument(
  mockSessionId: json['mockSessionId'] as String?,
  isQrCode: json['isQrCode'] as bool,
  isRefreshFlow: json['isRefreshFlow'] as bool? ?? false,
  uri: json['uri'] as String?,
);

Map<String, dynamic> _$IssuanceScreenArgumentToJson(
  _IssuanceScreenArgument instance,
) => <String, dynamic>{
  'mockSessionId': instance.mockSessionId,
  'isQrCode': instance.isQrCode,
  'isRefreshFlow': instance.isRefreshFlow,
  'uri': instance.uri,
};
