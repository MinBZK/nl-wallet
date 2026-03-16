import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/util/mapper/card/attribute/claim_display_metadata_mapper.dart';
import 'package:wallet_core/core.dart';

void main() {
  late ClaimDisplayMetadataMapper mapper;

  setUp(() {
    mapper = ClaimDisplayMetadataMapper();
  });

  group('ClaimDisplayMetadataMapper', () {
    test('should map list of ClaimDisplayMetadata to LocalizedText', () {
      final input = [
        const ClaimDisplayMetadata(lang: 'en', label: 'English Label'),
        const ClaimDisplayMetadata(lang: 'nl', label: 'Nederlands Label'),
      ];

      final result = mapper.map(input);

      expect(result, {
        const Locale('en'): 'English Label',
        const Locale('nl'): 'Nederlands Label',
      });
    });

    test('should handle empty list', () {
      final result = mapper.map([]);
      expect(result, <Locale, String>{});
    });

    test('should handle duplicate languages by taking a single, namely the last, one', () {
      final input = [
        const ClaimDisplayMetadata(lang: 'en', label: 'First'),
        const ClaimDisplayMetadata(lang: 'en', label: 'Second'),
      ];

      final result = mapper.map(input);

      expect(result, {
        const Locale('en'): 'Second',
      });
    });

    test('should parse complex locale strings', () {
      final input = [
        const ClaimDisplayMetadata(lang: 'en-US', label: 'US English'),
      ];

      final result = mapper.map(input);

      expect(result, {
        const Locale.fromSubtags(languageCode: 'en', countryCode: 'US'): 'US English',
      });
    });
  });
}
