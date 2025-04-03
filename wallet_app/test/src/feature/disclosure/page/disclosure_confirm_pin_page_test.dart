import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_confirm_pin_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('DisclosureConfirmPinPage light', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        DisclosureConfirmPinPage(
          title: l10n.disclosureConfirmPinPageTitle,
          bloc: PinBloc(Mocks.create()),
          onConfirmWithPinFailed: (context, state) {},
          onPinValidated: (_) {},
        ),
      );
      await screenMatchesGolden('disclosure_confirm_pin/light');
    });

    testGoldens('DisclosureConfirmPinPage light - landscape', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        DisclosureConfirmPinPage(
          title: l10n.disclosureConfirmPinPageTitle,
          bloc: PinBloc(Mocks.create()),
          onConfirmWithPinFailed: (context, state) {},
          onPinValidated: (_) {},
        ),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('disclosure_confirm_pin/light.landscape');
    });

    testGoldens('DisclosureConfirmPinPage dark', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        DisclosureConfirmPinPage(
          title: l10n.disclosureConfirmPinPageTitle,
          bloc: PinBloc(Mocks.create()),
          onConfirmWithPinFailed: (context, state) {},
          onPinValidated: (_) {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('disclosure_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('DisclosureConfirmPinPage renders the correct title', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        DisclosureConfirmPinPage(
          title: l10n.disclosureConfirmPinPageTitle,
          onPinValidated: (_) {},
          bloc: PinBloc(Mocks.create()),
          onConfirmWithPinFailed: (context, state) {},
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.disclosureConfirmPinPageTitle, findRichText: true);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
    });
  });
}
