// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'flutter_api_error.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

FlutterApiError _$FlutterApiErrorFromJson(Map<String, dynamic> json) => FlutterApiError(
      type: $enumDecode(_$FlutterApiErrorTypeEnumMap, json['type']),
      description: json['description'] as String?,
      data: json['data'] as Map<String, dynamic>?,
    );

Map<String, dynamic> _$FlutterApiErrorToJson(FlutterApiError instance) => <String, dynamic>{
      'type': _$FlutterApiErrorTypeEnumMap[instance.type]!,
      'description': instance.description,
      'data': instance.data,
    };

const _$FlutterApiErrorTypeEnumMap = {
  FlutterApiErrorType.generic: 'Generic',
  FlutterApiErrorType.networking: 'Networking',
  FlutterApiErrorType.walletState: 'WalletState',
  FlutterApiErrorType.hardwareKeyUnsupported: 'HardwareKeyUnsupported',
  FlutterApiErrorType.redirectUri: 'RedirectUri',
  FlutterApiErrorType.disclosureSourceMismatch: 'DisclosureSourceMismatch',
};
