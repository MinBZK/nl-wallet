import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/feature/common/widget/history/wallet_event_row.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(350, 118);

  group('goldens', () {
    testGoldens(
      'light wallet_event operation issued',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.issuanceEvent,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.operation.issued');
      },
    );
    testGoldens(
      'dark wallet_event operation issued',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.issuanceEvent,
            onPressed: () {},
          ),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/dark.operation.issued');
      },
    );

    testGoldens(
      'light wallet_event interaction success',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.disclosureEvent,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.interaction.success');
      },
    );

    testGoldens(
      'light wallet_event interaction failed',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.failedDisclosureEvent,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.interaction.failed');
      },
    );

    testGoldens(
      'light wallet_event login failed',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.failedLoginEvent,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.login.failed');
      },
    );

    testGoldens(
      'light wallet_event interaction failed - no data shared',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.failedDisclosureEventNothingShared,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.interaction.failed.no_data_shared');
      },
    );

    testGoldens(
      'light wallet_event login failed - no data shared',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.failedLoginEventNothingShared,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.login.failed.no_data_shared');
      },
    );

    testGoldens(
      'light wallet_event interaction rejected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletEvent.disclosure(
              dateTime: DateTime(2024),
              status: EventStatus.cancelled,
              relyingParty: WalletMockData.organization,
              purpose: 'disclosure'.untranslated,
              cards: [WalletMockData.card],
              policy: WalletMockData.policy,
              type: DisclosureType.regular,
            ),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.interaction.rejected');
      },
    );

    testGoldens(
      'light wallet_event signing success',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletMockData.signEvent,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.signing.success');
      },
    );
    testGoldens(
      'light wallet_event signing rejected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletEventRow(
            event: WalletEvent.sign(
              dateTime: DateTime(2024),
              status: EventStatus.cancelled,
              relyingParty: WalletMockData.organization,
              policy: WalletMockData.policy,
              document: WalletMockData.document,
            ),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('wallet_event_row/light.signing.rejected');
      },
    );
  });

  group('widgets', () {
    testWidgets('onPressed is triggered', (tester) async {
      bool tapped = false;
      await tester.pumpWidgetWithAppWrapper(
        WalletEventRow(
          event: WalletMockData.issuanceEvent,
          onPressed: () => tapped = true,
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text(WalletMockData.issuanceEvent.card.title.testValue);
      expect(titleFinder, findsOneWidget);

      // Tap any title, as the whole row should be clickable
      await tester.tap(titleFinder.last);
      expect(tapped, true, reason: 'onPressed was not called');
    });
  });
}
