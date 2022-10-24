import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:wallet/src/feature/theme/theme_screen.dart';

import 'feature/dashboard/dashboard_screen.dart';
import 'feature/pin/bloc/pin_bloc.dart';
import 'feature/pin/pin_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  static const splashRoute = '/';
  static const pinRoute = '/pin';
  static const dashboardRoute = '/dashboard';
  static const themeRoute = '/theme';

  static Route<dynamic> routeFactory(RouteSettings settings) {
    switch (settings.name) {
      case WalletRoutes.splashRoute:
        return MaterialPageRoute(builder: _createSplashRoute);
      case WalletRoutes.pinRoute:
        return MaterialPageRoute(builder: _createPinRoute);
      case WalletRoutes.dashboardRoute:
        return MaterialPageRoute(builder: _createDashboardRoute);
      case WalletRoutes.themeRoute:
        return MaterialPageRoute(builder: _createThemeRoute);
    }
    throw UnsupportedError('Unknown route: ${settings.name}');
  }

  static List<Route<dynamic>> initialRoutes(String route) => [MaterialPageRoute(builder: _createSplashRoute)];
}

Widget _createSplashRoute(BuildContext context) => BlocProvider<SplashBloc>(
      create: (BuildContext context) => SplashBloc(context.read()),
      child: const SplashScreen(),
    );

Widget _createPinRoute(BuildContext context) => BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read(), context.read()),
      child: const PinScreen(),
    );

Widget _createDashboardRoute(BuildContext context) => const DashboardScreen();

Widget _createThemeRoute(BuildContext context) => const ThemeScreen();
