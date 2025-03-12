import 'dart:ui';

import 'package:collection/collection.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/history/detail/request_details_screen.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';

void main() {
  group('widgets', () {
    testWidgets("The event's attributes labels are shown", (tester) async {
      final event = WalletMockData.disclosureEvent;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen(event: event)
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      final allAttributes = event.cards.map((card) => card.attributes).flattened;
      final allLabels = allAttributes.map((attribute) => attribute.label.testValue);
      for (final label in allLabels) {
        expect(find.text(label), findsOneWidget);
      }
    });

    testWidgets("The event's purpose is shown", (tester) async {
      final event = WalletMockData.disclosureEvent;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen(event: event)
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      expect(find.text(event.purpose.testValue), findsOneWidget);
    });
  });

  group('goldens', () {
    testGoldens('RequestDetails', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RequestDetailsScreen(event: WalletMockData.disclosureEvent)
                .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
            name: 'disclosure',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'request_details.light');
    });

    testGoldens('RequestDetails', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RequestDetailsScreen(event: WalletMockData.disclosureEvent)
                .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
            name: 'disclosure',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'request_details.dark');
    });
  });
}
