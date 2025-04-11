import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
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
    testGoldens('CardDataLoadSuccess Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.card),
        ),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('CardDataLoadSuccess Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.card),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('CardDataInitial state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataInitial(),
        ),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('CardDataLoadInProgress state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          const CardDataLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('CardDataLoadFailure state', (tester) async {
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
    testWidgets('card title is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CardDataScreen(cardTitle: 'Card Title').withState<CardDataBloc, CardDataState>(
          MockCardDataBloc(),
          CardDataLoadSuccess(WalletMockData.card),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;

      // Validate that the widget exists
      final titleFinder = find.text(l10n.cardDataScreenTitle(WalletMockData.card.title.testValue));
      final labelFinder = find.text(WalletMockData.textDataAttribute.label.l10nValueForLocale(Locale('en')));
      final valueFinder = find
          .text(AttributeValueFormatter.formatWithLocale(const Locale('en'), WalletMockData.textDataAttribute.value));
      expect(titleFinder, findsAtLeastNWidgets(1));
      expect(labelFinder, findsOneWidget);
      expect(valueFinder, findsOneWidget);
    });

    testWidgets('error is rendered when card cant be loaded', (tester) async {
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
