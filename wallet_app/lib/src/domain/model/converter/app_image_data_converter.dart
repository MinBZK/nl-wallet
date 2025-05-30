import 'dart:convert';

import 'package:json_annotation/json_annotation.dart';

import '../app_image_data.dart';

const _kTypeKey = 'type';
const _kDataKey = 'data';

class AppImageDataConverter extends JsonConverter<AppImageData, Map<String, dynamic>> {
  const AppImageDataConverter();

  @override
  AppImageData fromJson(Map<String, dynamic> json) {
    final String type = json[_kTypeKey];
    return switch (type) {
      'asset' => AppAssetImage(json[_kDataKey]),
      'base64' => AppMemoryImage(const Base64Decoder().convert(json[_kDataKey])),
      'svg' => SvgImage(json[_kDataKey]),
      String() => throw UnsupportedError('could not deserialize to image'),
    };
  }

  @override
  Map<String, dynamic> toJson(AppImageData object) {
    return switch (object) {
      AppAssetImage() => {
          _kTypeKey: 'asset',
          _kDataKey: object.name,
        },
      AppMemoryImage() => {
          _kTypeKey: 'base64',
          _kDataKey: const Base64Encoder().convert(object.data),
        },
      SvgImage() => {
          _kTypeKey: 'svg',
          _kDataKey: object.data,
        },
    };
  }
}
