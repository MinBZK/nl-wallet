import 'dart:ui';

import 'package:fimber/fimber.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_data_incorrect_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeDataIncorrectScreen Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: WalletPersonalizeDataIncorrectScreen(onDataRejected: () => Fimber.d('Data rejected')),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize_data_incorrect/light');
    });

    testGoldens('WalletPersonalizeDataIncorrectScreen Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: WalletPersonalizeDataIncorrectScreen(onDataRejected: () => Fimber.d('Data rejected')),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'wallet_personalize_data_incorrect/dark');
    });
  });

  group('widgets', () {
    testWidgets('description is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeDataIncorrectScreen(onDataRejected: () => Fimber.d('Data rejected')),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeDataIncorrectScreenDescription), findsOneWidget);
    });
  });
}
