import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/card_front.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/card/card_front_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet/src/wallet_assets.dart';
import 'package:wallet_core/core.dart';

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../test_utils.dart';

const _kSampleIssuer = CoreMockData.organization;

void main() {
  late Mapper<Attestation, LocalizedText?> mockSubtitleMapper;

  late Mapper<Attestation, CardFront> mapper;

  setUp(() {
    mockSubtitleMapper = MockMapper();

    mapper = CardFrontMapper(mockSubtitleMapper);
  });

  group('map', () {
    test('attestation with `com.example.pid` attestationType should return light localized card front', () async {
      const coreCard = Attestation(
        identity: AttestationIdentity.ephemeral(),
        attestationType: 'com.example.pid',
        displayMetadata: [CoreMockData.enDisplayMetadata],
        attributes: [],
        issuer: _kSampleIssuer,
      );

      when(mockSubtitleMapper.map(coreCard)).thenReturn('Subtitle'.untranslated);

      final nlL10n = await TestUtils.dutchLocalizations;
      final enL10n = await TestUtils.englishLocalizations;

      final expected = CardFront(
        title: {'en': enL10n.pidIdCardTitle, 'nl': nlL10n.pidIdCardTitle},
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

    test('attestation with `com.example.address` attestationType should return dark localized card front', () {
      const coreCard = Attestation(
        identity: AttestationIdentity.ephemeral(),
        attestationType: 'com.example.address',
        displayMetadata: [CoreMockData.enDisplayMetadata],
        attributes: [],
        issuer: _kSampleIssuer,
      );

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

    test('attestation with unknown attestationType should throw exception', () {
      const input = Attestation(
        identity: AttestationIdentity.ephemeral(),
        attestationType: 'unknown',
        displayMetadata: [CoreMockData.enDisplayMetadata],
        attributes: [],
        issuer: _kSampleIssuer,
      );

      expect(() => mapper.map(input), throwsException);
    });
  });
}
