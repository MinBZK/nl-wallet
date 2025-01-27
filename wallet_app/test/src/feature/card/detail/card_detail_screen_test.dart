import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card_config.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/feature/card/detail/bloc/card_detail_bloc.dart';
import 'package:wallet/src/feature/card/detail/card_detail_screen.dart';
import 'package:wallet/src/feature/common/widget/card/wallet_card_item.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

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
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: CardDetailScreen(
              cardTitle: WalletMockData.card.front.title.testValue,
            ).withState<CardDetailBloc, CardDetailState>(
              MockCardSummaryBloc(),
              cardDetailLoadSuccessMock,
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('CardDetailLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: CardDetailScreen(
              cardTitle: WalletMockData.card.front.title.testValue,
            ).withState<CardDetailBloc, CardDetailState>(
              MockCardSummaryBloc(),
              cardDetailLoadSuccessMock,
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('CardDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.front.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          const CardDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('CardDetailLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.front.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadFailure(WalletMockData.card.id),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('card is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.front.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock,
        ),
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.front.title.testValue);
      expect(cardTitleFinder, findsNWidgets(3)); // App bar (collapsed and expanded) title + title on card
    });

    testWidgets('loading renders as expected, with title and loading indicator', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.front.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          const CardDetailLoadInProgress(),
        ),
      );

      final cardTitleFinder = find.text(WalletMockData.card.front.title.testValue);
      expect(cardTitleFinder, findsAtLeast(1));

      // Validate that the loader is shown
      final loadingIndicatorFinder = find.byType(CenteredLoadingIndicator);
      expect(loadingIndicatorFinder, findsOneWidget);
    });

    testWidgets('loading with card renders as expected, with title and card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.front.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadInProgress(card: WalletMockData.card),
        ),
      );

      // Find the card title
      final cardTitleFinder = find.text(WalletMockData.card.front.title.testValue);
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
          cardTitle: WalletMockData.card.front.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadFailure(WalletMockData.card.id),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final retryFinder = find.text(l10n.generalRetry);
      expect(retryFinder, findsOneWidget);
    });
  });

  testWidgets('update button is shown when card is update-able', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      CardDetailScreen(
        cardTitle: WalletMockData.card.front.title.testValue,
      ).withState<CardDetailBloc, CardDetailState>(
        MockCardSummaryBloc(),
        CardDetailLoadSuccess(
          WalletCardDetail(
            card: WalletCard(
              docType: 'com.example.docType',
              front: WalletMockData.cardFront,
              issuer: WalletMockData.organization,
              attributes: const [],
              id: 'id',
              config: const CardConfig(updatable: true),
            ),
            mostRecentIssuance: WalletMockData.issuanceEvent,
            mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
          ),
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    // Validate that the widget exists
    final ctaFinder = find.text(l10n.cardDetailScreenCardUpdateCta);
    expect(ctaFinder, findsOneWidget);
  });

  testWidgets('remove button is shown when card is remove-able', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      CardDetailScreen(
        cardTitle: WalletMockData.card.front.title.testValue,
      ).withState<CardDetailBloc, CardDetailState>(
        MockCardSummaryBloc(),
        CardDetailLoadSuccess(
          WalletCardDetail(
            card: WalletCard(
              docType: 'com.example.docType',
              front: WalletMockData.cardFront,
              issuer: WalletMockData.organization,
              attributes: const [],
              id: 'id',
              config: const CardConfig(removable: true),
            ),
            mostRecentIssuance: WalletMockData.issuanceEvent,
            mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
          ),
        ),
      ),
    );

    final l10n = await TestUtils.englishLocalizations;
    // Validate that the widget exists
    final ctaFinder = find.text(l10n.cardDetailScreenCardDeleteCta);
    expect(ctaFinder, findsOneWidget);
  });
}
