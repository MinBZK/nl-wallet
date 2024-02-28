import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_confirm_pin_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('DisclosureConfirmPinPage light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: DisclosureConfirmPinPage(
              bloc: PinBloc(Mocks.create()),
              onConfirmWithPinFailed: (context, state) {},
              onPinValidated: (_) {},
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'disclosure_confirm_pin/light');
    });

    testGoldens('DisclosureConfirmPinPage dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: DisclosureConfirmPinPage(
              bloc: PinBloc(Mocks.create()),
              onConfirmWithPinFailed: (context, state) {},
              onPinValidated: (_) {},
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'disclosure_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('DisclosureConfirmPinPage renders the correct title & subtitle', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: DisclosureConfirmPinPage(
            onPinValidated: (_) {},
            bloc: PinBloc(Mocks.create()),
            onConfirmWithPinFailed: (context, state) {},
          ),
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.disclosureConfirmPinPageTitle);
      final descriptionFinder = find.text(l10n.disclosureConfirmPinPageDescription);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
