import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/util/mapper/card/card_subtitle_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet_core/core.dart';

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

const _kSampleAttributeName = CoreMockData.attestationAttributeName;
const _kSampleAttributeCity = CoreMockData.attestationAttributeCity;

const _kSampleNameSubtitle = {'en': 'Willeke', 'nl': 'Willeke'};
const _kSampleCitySubtitle = {'en': 'Den Haag', 'nl': 'Den Haag'};
const _kSampleIssuer = CoreMockData.organization;

void main() {
  late Mapper<AttestationValue, AttributeValue> mockAttributeValueMapper;

  late Mapper<Attestation, LocalizedText?> mapper;

  setUp(() {
    mockAttributeValueMapper = MockMapper();
    mapper = CardSubtitleMapper(mockAttributeValueMapper);
  });

  Attestation createSampleAttestation(String attestationType, List<AttestationAttribute> attributes) {
    return Attestation(
      identity: const AttestationIdentity.ephemeral(),
      attestationType: attestationType,
      displayMetadata: [CoreMockData.displayMetadata],
      attributes: attributes,
      issuer: _kSampleIssuer,
    );
  }

  group('map', () {
    test('attestation with `com.example.pid` attestationType should return `name` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleAttributeName.value)).thenReturn(const StringValue('Willeke'));

      final input = createSampleAttestation('com.example.pid', [_kSampleAttributeName, _kSampleAttributeCity]);
      expect(mapper.map(input), _kSampleNameSubtitle);

      // Check if every supported locale is mapped to a value
      verify(mockAttributeValueMapper.map(_kSampleAttributeName.value))
          .called(AppLocalizations.supportedLocales.length);
    });

    test('attestation with `com.example.pid` docType should return `name` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleAttributeName.value)).thenReturn(const StringValue('Willeke'));

      final input = createSampleAttestation('com.example.pid', [_kSampleAttributeName, _kSampleAttributeCity]);
      expect(mapper.map(input), _kSampleNameSubtitle);

      // Check if every supported locale is mapped to a value
      verify(mockAttributeValueMapper.map(_kSampleAttributeName.value))
          .called(AppLocalizations.supportedLocales.length);
    });

    test('`com.example.pid` attestation without `name` attribute should not return any subtitle', () {
      final input = createSampleAttestation('com.example.pid', [_kSampleAttributeCity]);
      expect(mapper.map(input), null);

      verifyNever(mockAttributeValueMapper.map(_kSampleAttributeName.value));
    });

    test('attestation with `com.example.address` attestationType should return `city` attribute string', () {
      when(mockAttributeValueMapper.map(_kSampleAttributeCity.value)).thenReturn(const StringValue('Den Haag'));

      final input = createSampleAttestation('com.example.address', [_kSampleAttributeName, _kSampleAttributeCity]);
      expect(mapper.map(input), _kSampleCitySubtitle);

      // Check if every supported locale is mapped to a value
      verify(mockAttributeValueMapper.map(_kSampleAttributeCity.value))
          .called(AppLocalizations.supportedLocales.length);
    });

    test('`com.example.address` attestation without `city` attribute should not return any subtitle', () {
      final input = createSampleAttestation('com.example.address', [_kSampleAttributeName]);
      expect(mapper.map(input), null);

      verifyNever(mockAttributeValueMapper.map(_kSampleAttributeName.value));
    });

    test('attestation with unknown attestationType should not return any subtitle', () {
      final input = createSampleAttestation('invalid_doctype', [_kSampleAttributeName, _kSampleAttributeCity]);
      expect(mapper.map(input), null);
    });
  });
}
