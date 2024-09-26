import 'package:collection/collection.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/history/detail/request_details_screen.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';

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
}
