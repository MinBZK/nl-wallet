import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../card/overview/card_overview_screen.dart';
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
          return false; // Back gesture disabled for demo purposes
        },
      ),
      bottomNavigationBar: _buildBottomNavigationBar(context),
    );
  }

  Widget _buildBody() {
    return BlocBuilder<HomeBloc, HomeState>(
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
          currentIndex: state.tab.index,
          onTap: (value) {
            final homeTab = HomeTab.values[value];
            context.read<HomeBloc>().add(HomeTabPressed(homeTab));
          },
          items: items,
        );
      },
    );
  }
}
