import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../card/overview/card_overview_screen.dart';
import '../menu/bloc/menu_bloc.dart';
import '../menu/menu_screen.dart';
import '../qr/qr_screen.dart';
import 'bloc/home_bloc.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: WillPopScope(
        child: _buildBody(),
        onWillPop: () async {
          if ((context.read<HomeBloc>().state.tab) == HomeTab.menu) {
            context.read<MenuBloc>().add(MenuHomePressed());
          }
          return false; // Back gesture disabled for demo purposes
        },
      ),
      bottomNavigationBar: _buildBottomNavigationBar(context),
    );
  }

  Widget _buildBody() {
    return BlocConsumer<HomeBloc, HomeState>(
      listenWhen: (prev, current) => prev.tab == current.tab,
      listener: (context, state) {
        if (state.tab == HomeTab.menu) context.read<MenuBloc>().add(MenuHomePressed());
      },
      builder: (context, state) {
        final Widget tab;
        switch (state.tab) {
          case HomeTab.cards:
            tab = const CardOverviewScreen();
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
      BottomNavigationBarItem(icon: const Icon(Icons.credit_card), label: context.l10n.homeScreenBottomNavBarCardsCta),
      BottomNavigationBarItem(icon: const Icon(Icons.qr_code), label: context.l10n.homeScreenBottomNavBarQrCta),
      BottomNavigationBarItem(icon: const Icon(Icons.menu), label: context.l10n.homeScreenBottomNavBarMenuCta),
    ];

    return BlocBuilder<HomeBloc, HomeState>(
      builder: (context, state) {
        return BottomNavigationBar(
          currentIndex: state.tab.tabIndex,
          onTap: (value) {
            final homeTab = HomeTab.values.firstWhereOrNull((element) => element.tabIndex == value);
            final forceStateRefresh = state.tab == HomeTab.menu;
            if (homeTab != null) {
              context.read<HomeBloc>().add(HomeTabPressed(homeTab, forceStateRefresh: forceStateRefresh));
            }
          },
          items: items,
        );
      },
    );
  }
}
