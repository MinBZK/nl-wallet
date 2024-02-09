import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../navigation/wallet_routes.dart';
import '../dashboard/bloc/dashboard_bloc.dart';
import '../dashboard/dashboard_screen.dart';
import '../menu/menu_screen.dart';
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
          case HomeTab.menu:
            tab = const MenuScreen();
            break;
        }
        return SafeArea(child: tab);
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
