import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/formatter/wallet_event_title_formatter.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';

void main() {
  group('WalletEventTitleFormatter', () {
    testWidgets('DeletionEvent returns the card title', (tester) async {
      late String formatted;
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            formatted = WalletEventTitleFormatter.format(context, WalletMockData.deletionEvent);
            return const SizedBox.shrink();
          },
        ),
      );
      expect(formatted, WalletMockData.card.title.testValue);
    });

    testWidgets('DisclosureEvent returns the relying party display name', (tester) async {
      late String formatted;
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            formatted = WalletEventTitleFormatter.format(context, WalletMockData.disclosureEvent);
            return const SizedBox.shrink();
          },
        ),
      );
      expect(formatted, WalletMockData.organization.displayName.testValue);
    });

    testWidgets('IssuanceEvent returns the card title', (tester) async {
      late String formatted;
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            formatted = WalletEventTitleFormatter.format(context, WalletMockData.issuanceEvent);
            return const SizedBox.shrink();
          },
        ),
      );
      expect(formatted, WalletMockData.card.title.testValue);
    });

    testWidgets('SignEvent returns the relying party display name', (tester) async {
      late String formatted;
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            formatted = WalletEventTitleFormatter.format(context, WalletMockData.signEvent);
            return const SizedBox.shrink();
          },
        ),
      );
      expect(formatted, WalletMockData.organization.displayName.testValue);
    });
  });
}
