import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/util/mapper/image/image_mapper.dart';
import 'package:wallet_core/core.dart';

void main() {
  late ImageMapper mapper;

  setUp(() {
    mapper = ImageMapper();
  });

  test('validate svg mapper', () {
    const input = Image.svg(xml: 'xml');
    const expectedOutput = SvgImage('xml');
    expect(mapper.map(input), expectedOutput);
  });

  test('validate png mapper', () {
    const input = Image.png(base64: 'png');
    const expectedOutput = Base64Image('png');
    expect(mapper.map(input), expectedOutput);
  });

  test('validate jpg mapper', () {
    const input = Image.jpg(base64: 'jpg');
    const expectedOutput = Base64Image('jpg');
    expect(mapper.map(input), expectedOutput);
  });

  test('validate asset mapper', () {
    const input = Image.asset(path: '/path');
    const expectedOutput = AppAssetImage('/path');
    expect(mapper.map(input), expectedOutput);
  });
}
