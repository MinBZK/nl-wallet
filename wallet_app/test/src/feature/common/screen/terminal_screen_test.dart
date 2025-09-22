import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/screen/terminal_screen.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/secondary_button.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('TerminalScreen - general error', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        TerminalScreen(
          title: 'Error Title',
          description: 'This is a test error description.',
          illustration: WalletAssets.svg_error_general,
          primaryButton: PrimaryButton(
            text: Text(l10n.generalClose),
            icon: const Icon(Icons.close_outlined),
            onPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('terminal/general_error');
    });

    testGoldens('TerminalScreen - with secondary button', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        TerminalScreen(
          title: 'Another Title',
          description: 'Another description for testing purposes.',
          illustration: WalletAssets.svg_pin_forgot,
          primaryButton: PrimaryButton(
            text: Text(l10n.generalOkCta),
            onPressed: () {},
          ),
          secondaryButton: SecondaryButton(
            text: Text(l10n.generalCancelCta),
            icon: const Icon(Icons.close_outlined),
            onPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('terminal/with_secondary_button');
    });

    testGoldens('TerminalScreen - with secondary button - dark - landscape', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        TerminalScreen(
          title: 'Another Title',
          description: 'Another description for testing purposes.',
          illustration: WalletAssets.svg_pin_forgot,
          primaryButton: PrimaryButton(
            text: Text(l10n.generalOkCta),
            onPressed: () {},
          ),
          secondaryButton: SecondaryButton(
            text: Text(l10n.generalCancelCta),
            icon: const Icon(Icons.close_outlined),
            onPressed: () {},
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('terminal/with_secondary_button.dark.landscape');
    });
  });
}
