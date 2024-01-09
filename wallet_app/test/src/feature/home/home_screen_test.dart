import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/card/overview/bloc/card_overview_bloc.dart';
import 'package:wallet/src/feature/common/widget/sliver_wallet_app_bar.dart';
import 'package:wallet/src/feature/home/bloc/home_bloc.dart';
import 'package:wallet/src/feature/home/home_screen.dart';
import 'package:wallet/src/feature/menu/bloc/menu_bloc.dart';
import 'package:wallet/src/feature/qr/tab/qr_scan/bloc/qr_scan_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

class MockHomeBloc extends MockBloc<HomeEvent, HomeState> implements HomeBloc {}

class MockCardOverviewBloc extends MockBloc<CardOverviewEvent, CardOverviewState> implements CardOverviewBloc {}

class MockMenuBloc extends MockBloc<MenuEvent, MenuState> implements MenuBloc {}

class MockQrScanBloc extends MockBloc<QrScanEvent, QrScanState> implements QrScanBloc {}

void main() {
  final List<BlocProvider> providers = [
    BlocProvider<CardOverviewBloc>(create: (context) {
      var bloc = MockCardOverviewBloc();
      whenListen(
        bloc,
        Stream.value(const CardOverviewLoadInProgress()),
        initialState: const CardOverviewLoadInProgress(),
      );
      return bloc;
    }),
    BlocProvider<QrScanBloc>(create: (context) {
      var bloc = MockQrScanBloc();
      whenListen(
        bloc,
        Stream.value(const QrScanNoPermission(true)),
        initialState: const QrScanNoPermission(true),
      );
      return bloc;
    }),
    BlocProvider<MenuBloc>(create: (context) {
      var bloc = MockMenuBloc();
      whenListen(
        bloc,
        Stream.value(const MenuLoadSuccess(name: 'John Doe')),
        initialState: const MenuLoadSuccess(name: 'John Doe'),
      );
      return bloc;
    }),
  ];

  group('goldens', () {
    testGoldens('Cards Tab', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HomeScreen().withState<HomeBloc, HomeState>(
              MockHomeBloc(),
              const HomeScreenSelect(HomeTab.cards),
            ),
            name: 'cards',
          ),
        wrapper: walletAppWrapper(providers: providers),
      );
      await screenMatchesGolden(tester, 'card_tab.light');
    });

    testGoldens('Qr Tab', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HomeScreen().withState<HomeBloc, HomeState>(
              MockHomeBloc(),
              const HomeScreenSelect(HomeTab.qr),
            ),
            name: 'qr',
          ),
        wrapper: walletAppWrapper(providers: providers),
      );
      await screenMatchesGolden(tester, 'qr_tab.light');
    });

    testGoldens('Menu Tab', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HomeScreen().withState<HomeBloc, HomeState>(
              MockHomeBloc(),
              const HomeScreenSelect(HomeTab.menu),
            ),
            name: 'menu',
          ),
        wrapper: walletAppWrapper(providers: providers),
      );
      await screenMatchesGolden(tester, 'menu_tab.light');
    });

    testGoldens('Qr Tab Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HomeScreen().withState<HomeBloc, HomeState>(
              MockHomeBloc(),
              const HomeScreenSelect(HomeTab.qr),
            ),
            name: 'qr',
          ),
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
          providers: providers,
        ),
      );
      await screenMatchesGolden(tester, 'qr_tab.dark');
    });
  });

  group('widgets', () {
    testWidgets('clicking tabs updates the content', (tester) async {
      final homeBloc = HomeBloc();
      final homeBlocProvider = BlocProvider<HomeBloc>(create: (c) => homeBloc);
      await tester.pumpWidgetBuilder(
        const HomeScreen(),
        wrapper: walletAppWrapper(providers: [homeBlocProvider, ...providers]),
      );

      // Find the Appbar Title Widget
      final appBarFinder = find.byType(AppBar);
      final titleFinder = find.descendant(of: appBarFinder, matching: find.byType(Text));
      expect(titleFinder, findsOneWidget);
      var titleWidget = (titleFinder.evaluate().single.widget as Text);

      // Expect it to start at on the cards tab with `My cards` as title
      expect(titleWidget.data, 'My cards');

      // Tab the QR tab and verify that the page updated
      await tester.tap(find.text('QR'));
      await tester.pumpAndSettle();
      // We use `first` here because the appbar contains the QR tabs too.
      titleWidget = (titleFinder.evaluate().first.widget as Text);
      expect(titleWidget.data, 'QR-code');

      // Tab the Menu tab and verify that the page updated
      await tester.tap(find.text('Menu'));
      await tester.pumpAndSettle();
      // Menu page already uses the [SliverWalletAppBar], lookup accordingly
      final sliverWalletAppbarFinder = find.byType(SliverWalletAppBar);
      final titlesFinder = find.descendant(of: sliverWalletAppbarFinder, matching: find.byType(Text));
      final titleCandidates = titlesFinder.evaluate();
      expect(titleCandidates.length, 2, reason: 'SliverWalletAppBar should contain collapsed and expanded titles');
      for (final candidate in titleCandidates) {
        expect((candidate.widget as Text).data, 'Menu');
      }
    });
  });
}
