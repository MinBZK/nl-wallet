import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_setup_failed_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeSetupFailedScreen Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeSetupFailedScreen(),
      );
      await screenMatchesGolden('wallet_personalize_setup_failed/light');
    });

    testGoldens('WalletPersonalizeSetupFailedScreen Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeSetupFailedScreen(),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('wallet_personalize_setup_failed/dark');
    });
  });

  group('widgets', () {
    testWidgets('description is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const WalletPersonalizeSetupFailedScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSetupFailedScreenDescription), findsOneWidget);
    });

    testWidgets('cta is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const WalletPersonalizeSetupFailedScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSetupFailedScreenCta), findsOneWidget);
    });
  });
}
