import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/feature/settings/settings_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('SettingsScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('SettingsScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
        ],
      );
      await screenMatchesGolden('dark');
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
