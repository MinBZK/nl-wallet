// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'disclosure_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_DisclosureScreenArgument _$DisclosureScreenArgumentFromJson(
  Map<String, dynamic> json,
) => _DisclosureScreenArgument(
  type: DisclosureConnectionType.fromJson(json['type'] as Map<String, dynamic>),
);

Map<String, dynamic> _$DisclosureScreenArgumentToJson(
  _DisclosureScreenArgument instance,
) => <String, dynamic>{'type': instance.type.toJson()};

RemoteDisclosure _$RemoteDisclosureFromJson(Map<String, dynamic> json) => RemoteDisclosure(
  json['uri'] as String,
  isQrCode: json['isQrCode'] as bool,
  $type: json['runtimeType'] as String?,
);

Map<String, dynamic> _$RemoteDisclosureToJson(RemoteDisclosure instance) => <String, dynamic>{
  'uri': instance.uri,
  'isQrCode': instance.isQrCode,
  'runtimeType': instance.$type,
};

CloseProximityDisclosure _$CloseProximityDisclosureFromJson(
  Map<String, dynamic> json,
) => CloseProximityDisclosure($type: json['runtimeType'] as String?);

Map<String, dynamic> _$CloseProximityDisclosureToJson(
  CloseProximityDisclosure instance,
) => <String, dynamic>{'runtimeType': instance.$type};
