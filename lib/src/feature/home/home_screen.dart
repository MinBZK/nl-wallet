import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../dashboard/dashboard_screen.dart';
import '../qr/qr_screen.dart';
import '../setting/setting_screen.dart';
import 'bloc/home_bloc.dart';

class HomeScreen extends StatelessWidget {
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
        switch (state.screenIndex) {
          case 0:
            return const DashboardScreen();
          case 1:
            return const QrScreen();
          case 2:
            return const SettingScreen();
          default:
            return const DashboardScreen();
        }
      },
    );
  }

  Widget _buildBottomNavigationBar(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final items = [
      BottomNavigationBarItem(icon: const Icon(Icons.credit_card), label: locale.homeBottomNavBarCardsButton),
      BottomNavigationBarItem(icon: const Icon(Icons.qr_code), label: locale.homeBottomNavBarQrButton),
      BottomNavigationBarItem(icon: const Icon(Icons.settings_outlined), label: locale.homeBottomNavBarSettingsButton),
    ];

    return BlocBuilder<HomeBloc, HomeState>(
      builder: (context, state) {
        return BottomNavigationBar(
          currentIndex: _resolveBottomNavigationBarCurrentIndex(state),
          onTap: (value) => context.read<HomeBloc>().add(HomeTabPressed(value)),
          items: items,
        );
      },
    );
  }

  int _resolveBottomNavigationBarCurrentIndex(HomeState state) {
    if (state is HomeScreenSelect) return state.screenIndex;
    return 0;
  }
}
