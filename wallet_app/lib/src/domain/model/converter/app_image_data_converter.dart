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
      'base64' => Base64Image(json[_kDataKey]),
      'svg' => SvgImage(json[_kDataKey]),
      String() => throw UnsupportedError('could not deserialize to image'),
    };
  }

  @override
  Map<String, dynamic> toJson(AppImageData object) {
    final type = switch (object) {
      AppAssetImage() => 'asset',
      Base64Image() => 'base64',
      SvgImage() => 'svg',
    };
    return {
      _kTypeKey: type,
      _kDataKey: object.data,
    };
  }
}
