import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/menu/bloc/menu_bloc.dart';
import 'package:wallet/src/feature/menu/menu_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockMenuBloc extends MockBloc<MenuEvent, MenuState> implements MenuBloc {}

void main() {
  group('goldens', () {
    testGoldens('ltc26 light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MenuScreen().withState<MenuBloc, MenuState>(
          MockMenuBloc(),
          const MenuInitial(),
        ),
        providers: [
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );

      await screenMatchesGolden('light');
    });

    testGoldens('ltc26 dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MenuScreen().withState<MenuBloc, MenuState>(
          MockMenuBloc(),
          const MenuInitial(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );

      await screenMatchesGolden('dark.landscape');
    });
  });

  group('widgets', () {
    testWidgets('ltc26 expected menu items are visible', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MenuScreen(showDesignSystemRow: true).withState<MenuBloc, MenuState>(
          MockMenuBloc(),
          const MenuInitial(),
        ),
        providers: [
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.menuScreenTourCta), findsOneWidget);
      expect(find.text(l10n.menuScreenHelpCta), findsOneWidget);
      expect(find.text(l10n.menuScreenScanQrCta), findsOneWidget);
      expect(find.text(l10n.menuScreenHistoryCta), findsOneWidget);
      expect(find.text(l10n.menuScreenSettingsCta), findsOneWidget);
      expect(find.text(l10n.menuScreenFeedbackCta), findsOneWidget);
      expect(find.text(l10n.menuScreenAboutCta), findsOneWidget);
      expect(find.text(l10n.menuScreenDesignCta), findsOneWidget);
    });

    testWidgets('ltc26 design system menu item is hidden when disabled', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MenuScreen(showDesignSystemRow: false).withState<MenuBloc, MenuState>(
          MockMenuBloc(),
          const MenuInitial(),
        ),
        providers: [
          Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
        ],
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.menuScreenDesignCta), findsNothing);
    });
  });
}
