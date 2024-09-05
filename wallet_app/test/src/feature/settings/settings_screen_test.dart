import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/feature/settings/settings_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('SettingsScreen light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SettingsScreen(),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('SettingsScreen dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const SettingsScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('clicking change language navigates to the correct route', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;

      final buttonFinder = find.text(l10n.settingsScreenChangeLanguageCta);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.changeLanguageRoute, findRichText: true), findsOneWidget);
    });
  });
}
