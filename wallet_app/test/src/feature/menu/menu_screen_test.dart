import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/menu/bloc/menu_bloc.dart';
import 'package:wallet/src/feature/menu/menu_screen.dart';
import 'package:wallet/src/wallet_constants.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockMenuBloc extends MockBloc<MenuEvent, MenuState> implements MenuBloc {}

const _kUrlLauncherMethodChannel = MethodChannel('plugins.flutter.io/url_launcher');

void main() {
  group('goldens', () {
    testGoldens('ltc26 light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MenuScreen(showDesignSystemRow: false).withState<MenuBloc, MenuState>(
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

    group('feedback', () {
      late List<MethodCall> launchCalls;

      setUp(() {
        launchCalls = [];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
          _kUrlLauncherMethodChannel,
          (MethodCall methodCall) async {
            launchCalls.add(methodCall);
            return true;
          },
        );
      });

      tearDown(() {
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
          _kUrlLauncherMethodChannel,
          null,
        );
      });

      Future<void> pumpAndTapFeedback(WidgetTester tester, Locale locale) async {
        await tester.pumpWidgetWithAppWrapper(
          const MenuScreen(showDesignSystemRow: false).withState<MenuBloc, MenuState>(
            MockMenuBloc(),
            const MenuInitial(),
          ),
          locale: locale,
          providers: [
            Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
          ],
        );

        final l10n = await TestUtils.getLocalizations(locale);
        await tester.tap(find.text(l10n.menuScreenFeedbackCta));
        await tester.pumpAndSettle();
      }

      testWidgets('ltc26 feedback opens the english form in the browser', (WidgetTester tester) async {
        await pumpAndTapFeedback(tester, const Locale('en'));

        final launch = launchCalls.single;
        expect(launch.arguments['url'], kFeedbackFormUrlEn);
        expect(launch.arguments['useWebView'], isFalse);
      });

      testWidgets('ltc26 feedback opens the dutch form in the browser', (WidgetTester tester) async {
        await pumpAndTapFeedback(tester, const Locale('nl'));

        final launch = launchCalls.single;
        expect(launch.arguments['url'], kFeedbackFormUrlNl);
        expect(launch.arguments['useWebView'], isFalse);
      });
    });
  });
}
