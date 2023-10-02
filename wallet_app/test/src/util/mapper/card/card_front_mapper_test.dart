import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/util/mapper/card/card_front_mapper.dart';
import 'package:wallet/src/util/mapper/locale_mapper.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../mocks/wallet_mocks.dart';

const _kSampleLocale = Locale('nl');

void main() {
  late LocaleMapper<Card, String> mockSubtitleMapper;

  late LocaleMapper<Card, CardFront> mapper;

  setUp(() {
    mockSubtitleMapper = MockLocaleMapper();

    mapper = CardFrontMapper(mockSubtitleMapper);
  });

  group('map', () {
    test('card with `pid_id` or `com.example.pid` docType should return light localized card front', () {
      const inputOne = Card(persistence: CardPersistence.inMemory(), docType: 'pid_id', attributes: []);
      const inputTwo = Card(persistence: CardPersistence.inMemory(), docType: 'com.example.pid', attributes: []);

      when(mockSubtitleMapper.map(_kSampleLocale, inputOne)).thenReturn('Subtitle');
      when(mockSubtitleMapper.map(_kSampleLocale, inputTwo)).thenReturn('Subtitle');

      const expected = CardFront(
        title: 'Persoonsgegevens',
        subtitle: 'Subtitle',
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_light,
        theme: CardFrontTheme.light,
      );

      expect(mapper.map(_kSampleLocale, inputOne), expected);
      expect(mapper.map(_kSampleLocale, inputTwo), expected);

      verify(mockSubtitleMapper.map(_kSampleLocale, inputOne)).called(1);
      verify(mockSubtitleMapper.map(_kSampleLocale, inputTwo)).called(1);
    });

    test('card with `pid_address` or `com.example.address` docType should return dark localized card front', () {
      const inputOne = Card(persistence: CardPersistence.inMemory(), docType: 'pid_address', attributes: []);
      const inputTwo = Card(persistence: CardPersistence.inMemory(), docType: 'com.example.address', attributes: []);

      when(mockSubtitleMapper.map(_kSampleLocale, inputOne)).thenReturn('Subtitle');
      when(mockSubtitleMapper.map(_kSampleLocale, inputTwo)).thenReturn('Subtitle');

      const expected = CardFront(
        title: 'Woonadres',
        subtitle: 'Subtitle',
        logoImage: WalletAssets.logo_card_rijksoverheid,
        holoImage: WalletAssets.svg_rijks_card_holo,
        backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
        theme: CardFrontTheme.dark,
      );

      expect(mapper.map(_kSampleLocale, inputOne), expected);
      expect(mapper.map(_kSampleLocale, inputTwo), expected);

      verify(mockSubtitleMapper.map(_kSampleLocale, inputOne)).called(1);
      verify(mockSubtitleMapper.map(_kSampleLocale, inputTwo)).called(1);
    });

    test('card with unknown docType should throw exception', () {
      const input = Card(persistence: CardPersistence.inMemory(), docType: 'unknown', attributes: []);

      expect(() => mapper.map(_kSampleLocale, input), throwsException);
    });
  });
}
