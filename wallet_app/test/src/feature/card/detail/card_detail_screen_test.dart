import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/feature/card/detail/argument/card_detail_screen_argument.dart';
import 'package:wallet/src/feature/card/detail/bloc/card_detail_bloc.dart';
import 'package:wallet/src/feature/card/detail/card_detail_screen.dart';
import 'package:wallet/src/feature/common/widget/card/wallet_card_item.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockCardSummaryBloc extends MockBloc<CardDetailEvent, CardDetailState> implements CardDetailBloc {}

void main() {
  final cardDetailLoadSuccessMock = CardDetailLoadSuccess(
    WalletCardDetail(
      card: WalletMockData.card,
      mostRecentIssuance: WalletMockData.issuanceEvent,
      mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
    ),
  );

  group('goldens', () {
    testGoldens('CardDetailLoadSuccess light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock,
        ),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('CardDetailLoadSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock,
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('CardDetailLoadSuccess - renewable card - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadSuccess(
            WalletCardDetail(
              card: WalletMockData.card,
              mostRecentIssuance: WalletMockData.issuanceEvent,
              mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
            ),
            showRenewOption: true,
          ),
        ),
      );
      await screenMatchesGolden('success.renewable.light');
    });

    testGoldens('CardDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          const CardDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('CardDetailLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadFailure(WalletMockData.card.attestationId!),
        ),
      );
      await screenMatchesGolden('error.light');
    });
  });

  group('widgets', () {
    testWidgets('card is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock,
        ),
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      expect(cardTitleFinder, findsNWidgets(2)); // Screen title + title on card
    });

    testWidgets('loading renders as expected, with title and loading indicator', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          const CardDetailLoadInProgress(),
        ),
      );

      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      expect(cardTitleFinder, findsAtLeast(1));

      // Validate that the loader is shown
      final loadingIndicatorFinder = find.byType(CenteredLoadingIndicator);
      expect(loadingIndicatorFinder, findsOneWidget);
    });

    testWidgets('loading with card renders as expected, with title and card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadInProgress(card: WalletMockData.card),
        ),
      );

      // Find the card title
      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      expect(cardTitleFinder, findsAtLeast(1));

      // Find the preview card
      final cardFinder = find.byType(WalletCardItem);
      expect(cardFinder, findsOneWidget);

      // Validate that the loader is shown
      final loadingIndicatorFinder = find.byType(CenteredLoadingIndicator);
      expect(loadingIndicatorFinder, findsOneWidget);
    });

    testWidgets('error renders with expected, with retry cta', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadFailure(WalletMockData.card.attestationId!),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final retryFinder = find.text(l10n.generalRetry);
      expect(retryFinder, findsOneWidget);
    });
  });

  group('unit', () {
    test('CardDetailScreenArgument can be extracted from RouteSettings', () async {
      final inputCard = WalletCard(
        attestationId: 'id',
        attestationType: 'com.example.docType',
        issuer: WalletMockData.organization,
        attributes: const [],
      );
      final CardDetailScreenArgument inputArgument = CardDetailScreenArgument(
        card: inputCard,
        cardId: inputCard.attestationId!,
        cardTitle: ''.untranslated,
      );
      final resultArgument = CardDetailScreen.getArgument(RouteSettings(arguments: inputArgument.toJson()));
      expect(resultArgument, inputArgument);
    });
  });
}
