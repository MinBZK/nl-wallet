import 'dart:typed_data';

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
    const input = Image.svg(xml: '<svg></svg>');
    const expectedOutput = SvgImage('<svg></svg>');
    expect(mapper.map(input), expectedOutput);
  });

  test('validate png mapper', () {
    final data = Uint8List.fromList([0x89, 0x50, 0x4E, 0x47]);
    final input = Image.png(data: data);
    final expectedOutput = AppMemoryImage(data);
    expect(mapper.map(input), expectedOutput);
  });

  test('validate jpeg mapper', () {
    final data = Uint8List.fromList([0xFF, 0xD8, 0xFF]);
    final input = Image.jpeg(data: data);
    final expectedOutput = AppMemoryImage(data);
    expect(mapper.map(input), expectedOutput);
  });

  test('validate asset mapper', () {
    const input = Image.asset(path: '/path');
    const expectedOutput = AppAssetImage('/path');
    expect(mapper.map(input), expectedOutput);
  });
}
