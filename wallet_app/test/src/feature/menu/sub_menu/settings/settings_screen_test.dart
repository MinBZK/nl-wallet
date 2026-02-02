import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/common/dialog/reset_wallet_dialog.dart';
import 'package:wallet/src/feature/menu/sub_menu/settings/settings_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ltc27 SettingsScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ltc27 SettingsScreen dark - landscape', (tester) async {
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
    testWidgets('ltc27 clicking change language navigates to the correct route', (tester) async {
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

    testWidgets('clicking change pin navigates to the correct route', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;

      final buttonFinder = find.text(l10n.settingsScreenChangePinCta);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.changePinRoute, findRichText: true), findsOneWidget);
    });

    testWidgets('clicking biometrics navigates to the correct route', (tester) async {
      final mockBiometricsUseCase = MockGetSupportedBiometricsUseCase();
      when(mockBiometricsUseCase.invoke()).thenAnswer((_) async => Biometrics.face);

      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>.value(value: mockBiometricsUseCase),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      await tester.pumpAndSettle();

      final buttonFinder = find.textContaining('face');
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.biometricsSettingsRoute, findRichText: true), findsOneWidget);
    });

    testWidgets('clicking transfer wallet navigates to the correct route', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;

      final buttonFinder = find.text(l10n.settingsScreenTransferWalletCta);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.walletTransferFaqRoute, findRichText: true), findsOneWidget);
    });

    testWidgets('clicking manage notifications navigates to the correct route', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;

      final buttonFinder = find.text(l10n.settingsScreenManageNotificationsCta);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.manageNotificationsRoute, findRichText: true), findsOneWidget);
    });

    testWidgets('clicking clear data shows reset wallet dialog', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;

      final buttonFinder = find.text(l10n.settingsScreenClearDataCta);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.byType(ResetWalletDialog), findsOneWidget);
      expect(find.text(l10n.resetWalletDialogTitle), findsOneWidget);
    });

    testWidgets('clicking review revocation code navigates to the correct route', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SettingsScreen(),
        providers: [
          RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => MockGetSupportedBiometricsUseCase()),
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;

      final buttonFinder = find.text(l10n.settingsScreenShowRevocationCodeCta);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.reviewRevocationCodeRoute, findRichText: true), findsOneWidget);
    });
  });
}
