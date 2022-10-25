import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'feature/dashboard/dashboard_screen.dart';
import 'feature/pin/bloc/pin_bloc.dart';
import 'feature/pin/pin_overlay.dart';
import 'feature/pin/pin_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';
import 'feature/theme/theme_screen.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  /// Routes in this list will be shown WITHOUT pin (wallet unlock) requirement
  @visibleForTesting
  static const publicRoutes = [splashRoute, pinRoute, themeRoute];

  static const splashRoute = '/';
  static const pinRoute = '/pin';
  static const dashboardRoute = '/dashboard';
  static const themeRoute = '/theme';

  static Route<dynamic> routeFactory(RouteSettings settings) {
    WidgetBuilder builder = _widgetBuilderFactory(settings);
    if (publicRoutes.contains(settings.name)) {
      return MaterialPageRoute(builder: builder);
    } else {
      return SecuredPageRoute(builder: builder);
    }
  }

  static WidgetBuilder _widgetBuilderFactory(RouteSettings settings) {
    switch (settings.name) {
      case WalletRoutes.splashRoute:
        return _createSplashRoute;
      case WalletRoutes.pinRoute:
        return _createPinRoute;
      case WalletRoutes.dashboardRoute:
        return _createDashboardRoute;
      case WalletRoutes.themeRoute:
        return _createThemeRoute;
      default:
        throw UnsupportedError('Unknown route: ${settings.name}');
    }
  }

  static List<Route<dynamic>> initialRoutes(String route) => [MaterialPageRoute(builder: _createSplashRoute)];
}

Widget _createSplashRoute(BuildContext context) => BlocProvider<SplashBloc>(
      create: (BuildContext context) => SplashBloc(context.read()),
      child: const SplashScreen(),
    );

Widget _createPinRoute(BuildContext context) => BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read(), context.read()),
      child: PinScreen(onUnlock: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.dashboardRoute)),
    );

Widget _createDashboardRoute(BuildContext context) => const DashboardScreen();

Widget _createThemeRoute(BuildContext context) => const ThemeScreen();

class SecuredPageRoute<T> extends MaterialPageRoute<T> {
  SecuredPageRoute({required WidgetBuilder builder}) : super(builder: (context) => PinOverlay(child: builder(context)));
}
