import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'feature/card/add/card_add_screen.dart';
import 'feature/card/data/card_data_screen.dart';
import 'feature/card/history/card_history_screen.dart';
import 'feature/card/summary/bloc/card_summary_bloc.dart';
import 'feature/card/summary/card_summary_screen.dart';
import 'feature/home/bloc/home_bloc.dart';
import 'feature/home/home_screen.dart';
import 'feature/pin/bloc/pin_bloc.dart';
import 'feature/pin/pin_overlay.dart';
import 'feature/pin/pin_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';
import 'feature/theme/theme_screen.dart';
import 'feature/verification/bloc/verification_bloc.dart';
import 'feature/verification/verification_screen.dart';
import 'feature/wallet/create/wallet_create_screen.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  /// Routes in this list will be shown WITHOUT pin (wallet unlock) requirement
  @visibleForTesting
  static const publicRoutes = [splashRoute, pinRoute, themeRoute, walletCreateRoute];

  static const splashRoute = '/';
  static const pinRoute = '/pin';
  static const walletCreateRoute = '/wallet/create';
  static const homeRoute = '/home';
  static const cardAddRoute = '/card/add';
  static const cardSummaryRoute = '/card/summary';
  static const cardDataRoute = '/card/data';
  static const cardHistoryRoute = '/card/history';
  static const themeRoute = '/theme';
  static const verificationRoute = '/verification';

  static Route<dynamic> routeFactory(RouteSettings settings) {
    WidgetBuilder builder = _widgetBuilderFactory(settings);
    if (publicRoutes.contains(settings.name)) {
      return MaterialPageRoute(builder: builder, settings: settings);
    } else {
      return SecuredPageRoute(builder: builder, settings: settings);
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
        return _createCardAddRoute;
      case WalletRoutes.cardSummaryRoute:
        return _createCardSummaryRoute(settings);
      case WalletRoutes.cardDataRoute:
        return _createCardDataRoute;
      case WalletRoutes.cardHistoryRoute:
        return _createCardHistoryRoute;
      case WalletRoutes.themeRoute:
        return _createThemeRoute;
      case WalletRoutes.verificationRoute:
        return _createVerificationRoute(settings);
      case WalletRoutes.walletCreateRoute:
        return _createWalletCreateRoute;
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

Widget _createCardAddRoute(BuildContext context) => const CardAddScreen();

WidgetBuilder _createCardSummaryRoute(RouteSettings settings) {
  return (context) {
    final String cardId = CardSummaryScreen.getArguments(settings);
    return BlocProvider<CardSummaryBloc>(
      create: (context) => CardSummaryBloc(context.read())..add(CardSummaryLoadTriggered(cardId)),
      child: const CardSummaryScreen(),
    );
  };
}

Widget _createCardDataRoute(BuildContext context) => const CardDataScreen();

Widget _createCardHistoryRoute(BuildContext context) => const CardHistoryScreen();

Widget _createThemeRoute(BuildContext context) => const ThemeScreen();

WidgetBuilder _createVerificationRoute(RouteSettings settings) {
  return (context) {
    return BlocProvider<VerificationBloc>(
      create: (BuildContext context) {
        final bloc = VerificationBloc(context.read());
        bloc.add(VerificationLoadRequested(VerificationScreen.getArguments(settings)));
        return bloc;
      },
      child: const VerificationScreen(),
    );
  };
}

Widget _createWalletCreateRoute(BuildContext context) => const WalletCreateScreen();

class SecuredPageRoute<T> extends MaterialPageRoute<T> {
  SecuredPageRoute({required WidgetBuilder builder, super.settings})
      : super(builder: (context) => PinOverlay(child: builder(context)));
}
