import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/card/summary_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  late Mapper<WalletCard, LocalizedText> mapper;

  setUp(() async {
    /// Needed for [DateFormat] to work
    await initializeDateFormatting();
    mapper = CardSummaryMapper();
  });

  group('map', () {
    test('card without summary results in empty LocalizedText', () {
      final input = WalletCard(
        docType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: [],
        id: 'id',
        metadata: [CardDisplayMetadata(language: Locale('en'), name: 'name')],
      );
      expect(mapper.map(input).testValue, '');
    });

    test('card without placeholders in summary results in identical contents in LocalizedText', () {
      const summary = 'No svgIds';
      final input = WalletCard(
        docType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: [WalletMockData.textDataAttribute],
        metadata: const [
          CardDisplayMetadata(language: Locale('en'), name: 'name', rawSummary: summary),
        ],
        id: 'id',
      );
      expect(mapper.map(input).testValue, summary);
    });

    test('card with placeholders in summary should result in summary with placeholders replaced', () {
      const summary = 'First: {{first}}, Second: {{second}}';
      final input = WalletCard(
        docType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: [
          DataAttribute.untranslated(
            key: 'first',
            label: 'First name',
            value: const StringValue('John'),
            sourceCardDocType: 'com.example.docType',
          ),
          DataAttribute.untranslated(
            key: 'second',
            label: 'Last name',
            value: const StringValue('Doe'),
            sourceCardDocType: 'com.example.docType',
          ),
        ],
        metadata: const [
          CardDisplayMetadata(language: Locale('en'), name: 'name', rawSummary: summary),
        ],
        id: 'id',
      );
      expect(mapper.map(input).testValue, 'First: John, Second: Doe');
    });

    test('placeholders should be replaced taking localization into account', () {
      final input = WalletCard(
        docType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: [
          DataAttribute(
            key: 'over18',
            label: {Locale('en'): 'Over 18', Locale('nl'): 'Ouder dan 18'},
            value: const BooleanValue(true),
            sourceCardDocType: 'com.example.docType',
          ),
        ],
        metadata: const [
          CardDisplayMetadata(
            language: Locale('en'),
            name: 'name',
            rawSummary: 'User is 18+ {{over18}}',
          ),
          CardDisplayMetadata(
            language: Locale('nl'),
            name: 'naam',
            rawSummary: 'Gebruiker is 18+ {{over18}}',
          ),
        ],
        id: 'id',
      );

      expect(mapper.map(input)[Locale('en')], 'User is 18+ Yes');
      expect(mapper.map(input)[Locale('nl')], 'Gebruiker is 18+ Ja');
    });

    test('placeholders without a corresponding value should be blanked', () {
      final input = WalletCard(
        docType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: [],
        metadata: const [
          CardDisplayMetadata(
            language: Locale('en'),
            name: 'name',
            rawSummary: 'Middle name: {{middle_name}}',
          ),
        ],
        id: 'id',
      );

      expect(mapper.map(input).testValue, 'Middle name: ');
    });

    test('Dates are formatted based on localization', () {
      final input = WalletCard(
        docType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: [
          DataAttribute(
            key: 'dob',
            label: ''.untranslated,
            value: DateValue(DateTime(2024, 10, 5)),
            sourceCardDocType: 'com.example.docType',
          ),
        ],
        metadata: const [
          CardDisplayMetadata(
            language: Locale('en'),
            name: '',
            rawSummary: 'Date {{dob}}',
          ),
          CardDisplayMetadata(
            language: Locale('ja_JP'),
            name: '',
            rawSummary: '日付 {{dob}}',
          ),
          CardDisplayMetadata(
            language: Locale('nl'),
            name: '',
            rawSummary: 'Datum {{dob}}',
          ),
        ],
        id: 'id',
      );

      expect(mapper.map(input)[Locale('en')], 'Date 10/5/2024');
      expect(mapper.map(input)[Locale('ja_JP')], '日付 2024/10/5');
      expect(mapper.map(input)[Locale('nl')], 'Datum 5-10-2024');
    });
  });
}
