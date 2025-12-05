import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/feature/card/data/bloc/card_data_bloc.dart';
import 'package:wallet/src/feature/card/data/card_data_screen.dart';
import 'package:wallet/src/util/formatter/attribute_value_formatter.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockCardDataBloc extends MockBloc<CardDataEvent, CardDataState> implements CardDataBloc {}

void main() {
  group('goldens', () {
    testGoldens('ltc25 CardDataLoadSuccess Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.card),
        ),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('ltc25 CardDataLoadSuccess Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.card),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - validSoon', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.validSoon)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.valid.soon');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - valid', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.valid)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.valid');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - expiresSoon', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.expiresSoon)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.expires.soon');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - expired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.expired)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.expired');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - revoked', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.revoked)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.revoked');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - corrupted', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.corrupted)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.corrupted');
    });

    testGoldens('ltc25 CardDataLoadSuccess status - unknown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.cardWithStatus(CardStatus.unknown)),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('status.unknown');
    });

    testGoldens('ltc25 CardDataInitial state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataInitial(),
        ),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('ltc25 CardDataLoadInProgress state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('ltc25 CardDataLoadFailure state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadFailure(),
        ),
      );
      await screenMatchesGolden('error.light');
    });
  });

  group('widgets', () {
    testWidgets('ltc25 card title is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.card),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;

      // Validate that the widget exists
      final titleFinder = find.text(l10n.cardDataScreenTitle(WalletMockData.card.title.testValue));
      final labelFinder = find.text(WalletMockData.textDataAttribute.label.l10nValueForLocale(const Locale('en')));
      final valueFinder = find.text(
        AttributeValueFormatter.formatWithLocale(const Locale('en'), WalletMockData.textDataAttribute.value),
      );
      expect(titleFinder, findsAtLeastNWidgets(1));
      expect(labelFinder, findsOneWidget);
      expect(valueFinder, findsOneWidget);
    });

    testWidgets('ltc25 error is rendered when card cant be loaded', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadFailure(),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;

      final errorFinder = find.text(l10n.errorScreenGenericDescription);
      final ctaFinder = find.text(l10n.generalRetry);
      expect(errorFinder, findsAtLeastNWidgets(1));
      expect(ctaFinder, findsOneWidget);
    });
  });
}
