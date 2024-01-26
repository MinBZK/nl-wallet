import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../dashboard/bloc/dashboard_bloc.dart';
import '../dashboard/dashboard_screen.dart';
import '../menu/menu_screen.dart';
import '../qr/qr_screen.dart';
import 'argument/home_screen_argument.dart';
import 'bloc/home_bloc.dart';

class HomeScreen extends StatelessWidget {
  static HomeScreenArgument? getArgument(RouteSettings settings) {
    final args = settings.arguments;
    if (args == null) return null;
    try {
      return HomeScreenArgument.fromJson(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      return null;
    }
  }

  const HomeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _buildBody(),
      bottomNavigationBar: _buildBottomNavigationBar(context),
    );
  }

  Widget _buildBody() {
    return BlocBuilder<HomeBloc, HomeState>(
      builder: (context, state) {
        final Widget tab;
        switch (state.tab) {
          case HomeTab.cards:
            tab = const DashboardScreen();
            break;
          case HomeTab.qr:
            tab = const QrScreen();
            break;
          case HomeTab.menu:
            tab = const MenuScreen();
            break;
        }
        return SafeArea(child: tab);
      },
    );
  }

  Widget _buildBottomNavigationBar(BuildContext context) {
    final items = [
      BottomNavigationBarItem(
        icon: const Icon(Icons.credit_card),
        label: context.l10n.homeScreenBottomNavBarCardsCta,
        tooltip: context.l10n.homeScreenBottomNavBarCardsCta,
      ),
      BottomNavigationBarItem(
        icon: const Icon(Icons.qr_code),
        label: context.l10n.homeScreenBottomNavBarQrCta,
        tooltip: context.l10n.homeScreenBottomNavBarQrCta,
      ),
      BottomNavigationBarItem(
        icon: const Icon(Icons.menu),
        label: context.l10n.homeScreenBottomNavBarMenuCta,
        tooltip: context.l10n.homeScreenBottomNavBarMenuCta,
      ),
    ];

    final indicatorWidth = context.mediaQuery.size.width / items.length;
    const indicatorHeight = 2.0;
    const dividerHeight = 1.0;

    return BlocBuilder<HomeBloc, HomeState>(
      builder: (context, state) {
        return Stack(
          children: [
            BottomNavigationBar(
              key: const Key('homeScreenBottomNavigationBar'),
              currentIndex: state.tab.index,
              onTap: (value) {
                final homeTab = HomeTab.values[value];
                context.read<HomeBloc>().add(HomeTabPressed(homeTab));
              },
              items: items,
            ),
            Container(
              height: dividerHeight,
              width: double.infinity,
              color: context.colorScheme.outlineVariant,
            ),
            AnimatedPositioned(
              top: dividerHeight,
              height: indicatorHeight,
              width: indicatorWidth,
              left: indicatorWidth * state.tab.index,
              duration: kDefaultAnimationDuration,
              child: Container(color: context.colorScheme.primary),
            ),
          ],
        );
      },
    );
  }

  /// Show the [HomeScreen], placing it at the root of the navigation stack. When [cards] are provided the
  /// nested [DashboardScreen]'s [DashboardBloc] is initialized with these cards, so that they are instantly
  /// available, e.g. useful when triggering Hero animations.
  static void show(BuildContext context, {List<WalletCard>? cards}) {
    if (cards != null) SecuredPageRoute.overrideDurationOfNextTransition(const Duration(milliseconds: 800));
    Navigator.restorablePushNamedAndRemoveUntil(
      context,
      WalletRoutes.homeRoute,
      ModalRoute.withName(WalletRoutes.splashRoute),
      arguments: cards == null ? null : HomeScreenArgument(cards: cards).toJson(),
    );
  }
}
