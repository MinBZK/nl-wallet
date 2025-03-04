import 'package:test/test.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/converter/app_image_data_converter.dart';

void main() {
  const AppImageDataConverter converter = AppImageDataConverter();

  test('asset', () {
    const image = AppAssetImage('asset');
    final json = converter.toJson(image);
    final decodedImage = converter.fromJson(json);
    expect(image, equals(decodedImage));
  });

  test('base64', () {
    const image = Base64Image('base64');
    final json = converter.toJson(image);
    final decodedImage = converter.fromJson(json);
    expect(image, equals(decodedImage));
  });

  test('svg', () {
    const image = SvgImage('SVG');
    final json = converter.toJson(image);
    final decodedImage = converter.fromJson(json);
    expect(image, equals(decodedImage));
  });

  test('decoding an unsupported type throws', () {
    expect(
      () => converter.fromJson({'type': 'non-existent-type'}),
      throwsA(isA<UnsupportedError>()),
    );
  });
}
