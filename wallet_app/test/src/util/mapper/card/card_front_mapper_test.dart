import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/card/card_front_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet/src/wallet_assets.dart';
import 'package:wallet_core/core.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late Mapper<Card, LocalizedText?> mockSubtitleMapper;

  late Mapper<Card, CardFront> mapper;

  setUp(() {
    mockSubtitleMapper = MockMapper();

    mapper = CardFrontMapper(mockSubtitleMapper);
  });

  group('map', () {
    test('card with `com.example.pid` docType should return light localized card front', () {
      const coreCard = Card(persistence: CardPersistence.inMemory(), docType: 'com.example.pid', attributes: []);

      when(mockSubtitleMapper.map(coreCard)).thenReturn('Subtitle'.untranslated);

      final expected = CardFront(
        title: const {'en': 'Personal data', 'nl': 'Persoonsgegevens'},
        subtitle: 'Subtitle'.untranslated,
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_light,
        theme: CardFrontTheme.light,
      );

      expect(mapper.map(coreCard), expected);
      expect(kPidDocType, 'com.example.pid');

      verify(mockSubtitleMapper.map(coreCard)).called(1);
    });

    test('card with `com.example.address` docType should return dark localized card front', () {
      const coreCard = Card(persistence: CardPersistence.inMemory(), docType: 'com.example.address', attributes: []);

      when(mockSubtitleMapper.map(coreCard)).thenReturn('Subtitle'.untranslated);

      final expected = CardFront(
        title: const {'en': 'Residential address', 'nl': 'Woonadres'},
        subtitle: 'Subtitle'.untranslated,
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
        theme: CardFrontTheme.dark,
      );

      expect(mapper.map(coreCard), expected);
      expect(kAddressDocType, 'com.example.address');

      verify(mockSubtitleMapper.map(coreCard)).called(1);
    });

    test('card with unknown docType should throw exception', () {
      const input = Card(persistence: CardPersistence.inMemory(), docType: 'unknown', attributes: []);

      expect(() => mapper.map(input), throwsException);
    });
  });
}
