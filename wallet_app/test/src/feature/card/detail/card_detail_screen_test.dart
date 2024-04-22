import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/feature/card/detail/bloc/card_detail_bloc.dart';
import 'package:wallet/src/feature/card/detail/card_detail_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';

class MockCardSummaryBloc extends MockBloc<CardDetailEvent, CardDetailState> implements CardDetailBloc {}

void main() {
  final cardDetailLoadSuccessMock = CardDetailLoadSuccess(
    WalletCardDetail(
      card: WalletMockData.card,
      latestIssuedOperation: WalletMockData.operationTimelineAttribute,
      latestSuccessInteraction: WalletMockData.interactionTimelineAttribute,
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
  });
}
