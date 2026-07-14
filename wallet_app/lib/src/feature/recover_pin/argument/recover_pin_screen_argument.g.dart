// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'recover_pin_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_RecoverPinScreenArgument _$RecoverPinScreenArgumentFromJson(
  Map<String, dynamic> json,
) => _RecoverPinScreenArgument(
  uri: json['uri'] as String?,
  isRecoveryFlow: json['isRecoveryFlow'] as bool? ?? false,
);

Map<String, dynamic> _$RecoverPinScreenArgumentToJson(
  _RecoverPinScreenArgument instance,
) => <String, dynamic>{
  'uri': instance.uri,
  'isRecoveryFlow': instance.isRecoveryFlow,
};
