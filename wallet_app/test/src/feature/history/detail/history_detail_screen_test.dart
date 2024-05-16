import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/history/detail/bloc/history_detail_bloc.dart';
import 'package:wallet/src/feature/history/detail/history_detail_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';

class MockHistoryDetailBloc extends MockBloc<HistoryDetailEvent, HistoryDetailState> implements HistoryDetailBloc {}

void main() {
  group('goldens', () {
    testGoldens('HistoryDetailLoadSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('HistoryDetailLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('HistoryDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('Error state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadFailure(),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('card details are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.issuanceEvent, [WalletMockData.card]),
        ),
      );

      // Validate that the card details are rendered
      expect(find.text(WalletMockData.cardFront.title.testValue), findsOneWidget);
      for (final attribute in WalletMockData.issuanceEvent.attributes) {
        expect(find.textContaining(attribute.label.testValue), findsOneWidget);
        expect(find.textContaining(attribute.value.toString()), findsOneWidget);
      }
    });
  });
}
