import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/card/card_front_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late Mapper<Card, LocalizedText?> mockSubtitleMapper;

  late Mapper<Card, CardFront> mapper;

  setUp(() {
    mockSubtitleMapper = MockMapper();

    mapper = CardFrontMapper(mockSubtitleMapper);
  });

  group('map', () {
    test('card with `pid_id` or `com.example.pid` docType should return light localized card front', () {
      const inputOne = Card(persistence: CardPersistence.inMemory(), docType: 'pid_id', attributes: []);
      const inputTwo = Card(persistence: CardPersistence.inMemory(), docType: 'com.example.pid', attributes: []);

      when(mockSubtitleMapper.map(inputOne)).thenReturn('Subtitle'.untranslated);
      when(mockSubtitleMapper.map(inputTwo)).thenReturn('Subtitle'.untranslated);

      final expected = CardFront(
        title: const {'en': 'Personal data', 'nl': 'Persoonsgegevens'},
        subtitle: 'Subtitle'.untranslated,
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_light,
        theme: CardFrontTheme.light,
      );

      expect(mapper.map(inputOne), expected);
      expect(mapper.map(inputTwo), expected);

      verify(mockSubtitleMapper.map(inputOne)).called(1);
      verify(mockSubtitleMapper.map(inputTwo)).called(1);
    });

    test('card with `pid_address` or `com.example.address` docType should return dark localized card front', () {
      const inputOne = Card(persistence: CardPersistence.inMemory(), docType: 'pid_address', attributes: []);
      const inputTwo = Card(persistence: CardPersistence.inMemory(), docType: 'com.example.address', attributes: []);

      when(mockSubtitleMapper.map(inputOne)).thenReturn('Subtitle'.untranslated);
      when(mockSubtitleMapper.map(inputTwo)).thenReturn('Subtitle'.untranslated);

      final expected = CardFront(
        title: const {'en': 'Residential address', 'nl': 'Woonadres'},
        subtitle: 'Subtitle'.untranslated,
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
        theme: CardFrontTheme.dark,
      );

      expect(mapper.map(inputOne), expected);
      expect(mapper.map(inputTwo), expected);

      verify(mockSubtitleMapper.map(inputOne)).called(1);
      verify(mockSubtitleMapper.map(inputTwo)).called(1);
    });

    test('card with unknown docType should throw exception', () {
      const input = Card(persistence: CardPersistence.inMemory(), docType: 'unknown', attributes: []);

      expect(() => mapper.map(input), throwsException);
    });
  });
}
