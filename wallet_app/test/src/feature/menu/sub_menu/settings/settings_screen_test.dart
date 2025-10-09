import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/menu/sub_menu/settings/settings_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('SettingsScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('SettingsScreen dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      await screenMatchesGolden('dark.landscape');
    });
  });

  group('widgets', () {
    testWidgets('clicking change language navigates to the correct route', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
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
