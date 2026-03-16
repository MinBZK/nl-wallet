import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/metadata/card_rendering.dart';
import 'package:wallet/src/util/mapper/card/metadata_mapper.dart';
import 'package:wallet/src/util/mapper/image/image_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  late DisplayMetadataMapper mapper;

  setUp(() {
    mapper = DisplayMetadataMapper(ImageMapper());
  });

  group('DisplayMetadataMapper', () {
    test('should map basic display metadata', () {
      const input = core.DisplayMetadata(
        lang: 'en',
        name: 'Test Card',
        description: 'Test Description',
        summary: 'Test Summary',
      );

      final result = mapper.map(input);

      expect(
        result,
        const CardDisplayMetadata(
          language: Locale('en'),
          name: 'Test Card',
          description: 'Test Description',
          rawSummary: 'Test Summary',
          rendering: null,
        ),
      );
    });

    test('should map simple rendering metadata', () {
      const input = core.DisplayMetadata(
        lang: 'nl',
        name: 'Naam',
        rendering: core.RenderingMetadata_Simple(
          backgroundColor: '#FFFFFF',
          textColor: '#000000',
        ),
      );

      final result = mapper.map(input);

      expect(
        result.rendering,
        const CardRendering.simple(
          bgColor: Color(0xFFFFFFFF),
          textColor: Color(0xFF000000),
        ),
      );
    });

    test('should map logo in rendering metadata', () {
      const input = core.DisplayMetadata(
        lang: 'en',
        name: 'Logo Card',
        rendering: core.RenderingMetadata_Simple(
          logo: core.ImageWithMetadata(
            image: core.Image_Asset(path: 'assets/logo.png'),
            altText: 'Logo Alt',
          ),
        ),
      );

      final result = mapper.map(input);

      expect(
        result.rendering,
        const CardRendering.simple(
          logo: AppAssetImage('assets/logo.png'),
          logoAltText: 'Logo Alt',
        ),
      );
    });

    test('should return null rendering for SvgTemplates', () {
      const input = core.DisplayMetadata(
        lang: 'en',
        name: 'SVG Template Card',
        rendering: core.RenderingMetadata_SvgTemplates(),
      );

      final result = mapper.map(input);

      expect(result.rendering, isNull);
    });

    test('should handle 8-digit hex colors', () {
      const input = core.DisplayMetadata(
        lang: 'en',
        name: 'Transparent Card',
        rendering: core.RenderingMetadata_Simple(
          backgroundColor: '#80FFFFFF',
        ),
      );

      final result = mapper.map(input);

      expect(
        (result.rendering! as SimpleCardRendering).bgColor,
        const Color(0x80FFFFFF),
      );
    });

    test('should return null for invalid hex colors', () {
      const input = core.DisplayMetadata(
        lang: 'en',
        name: 'Invalid Color Card',
        rendering: core.RenderingMetadata_Simple(
          backgroundColor: 'invalid',
        ),
      );

      final result = mapper.map(input);

      expect((result.rendering! as SimpleCardRendering).bgColor, isNull);
    });
  });
}
