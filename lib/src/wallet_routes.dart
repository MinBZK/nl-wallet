import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'feature/card/add/card_add_screen.dart';
import 'feature/card/summary/card_summary_screen.dart';
import 'feature/home/bloc/home_bloc.dart';
import 'feature/home/home_screen.dart';
import 'feature/pin/bloc/pin_bloc.dart';
import 'feature/pin/pin_overlay.dart';
import 'feature/pin/pin_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';
import 'feature/theme/theme_screen.dart';
import 'feature/verification/verification_screen.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  /// Routes in this list will be shown WITHOUT pin (wallet unlock) requirement
  @visibleForTesting
  static const publicRoutes = [splashRoute, pinRoute, themeRoute];

  static const splashRoute = '/';
  static const pinRoute = '/pin';
  static const homeRoute = '/home';
  static const cardAddRoute = '/card/add';
  static const cardSummaryRoute = '/card/summary';
  static const themeRoute = '/theme';
  static const verificationRoute = '/verification';

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
      case WalletRoutes.homeRoute:
        return _createHomeRoute;
      case WalletRoutes.cardAddRoute:
        return _createCardAdd;
      case WalletRoutes.cardSummaryRoute:
        return _createCardSummary;
      case WalletRoutes.themeRoute:
        return _createThemeRoute;
      case WalletRoutes.verificationRoute:
        return _createVerificationRoute;
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
      child: PinScreen(onUnlock: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute)),
    );

Widget _createHomeRoute(BuildContext context) => BlocProvider<HomeBloc>(
      create: (BuildContext context) => HomeBloc(),
      child: const HomeScreen(),
    );

Widget _createCardAdd(BuildContext context) => const CardAddScreen();

Widget _createCardSummary(BuildContext context) => const CardSummaryScreen();

Widget _createThemeRoute(BuildContext context) => const ThemeScreen();

Widget _createVerificationRoute(BuildContext context) => const VerificationScreen();

class SecuredPageRoute<T> extends MaterialPageRoute<T> {
  SecuredPageRoute({required WidgetBuilder builder}) : super(builder: (context) => PinOverlay(child: builder(context)));
}
