import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/issuance/page/issuance_confirm_pin_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Confirm page light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
        ),
      );
      await screenMatchesGolden('issuance_confirm_pin/light');
    });

    testGoldens('Confirm page dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('issuance_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('IssuanceConfirmPinPage renders the correct title & subtitle', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinPage(
          onPinValidated: (_) {},
          bloc: PinBloc(Mocks.create()),
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.issuanceConfirmPinPageTitle, findRichText: true);
      final descriptionFinder = find.text(l10n.issuanceConfirmPinPageDescription, findRichText: true);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
