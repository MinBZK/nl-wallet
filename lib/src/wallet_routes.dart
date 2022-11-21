import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'domain/usecase/pin/unlock_wallet_usecase.dart';
import 'feature/card/add/card_add_screen.dart';
import 'feature/card/data/bloc/card_data_bloc.dart';
import 'feature/card/data/card_data_screen.dart';
import 'feature/card/history/bloc/card_history_bloc.dart';
import 'feature/card/history/card_history_screen.dart';
import 'feature/card/summary/bloc/card_summary_bloc.dart';
import 'feature/card/summary/card_summary_screen.dart';
import 'feature/home/bloc/home_bloc.dart';
import 'feature/home/home_screen.dart';
import 'feature/issuance/bloc/issuance_bloc.dart';
import 'feature/issuance/issuance_screen.dart';
import 'feature/pin/bloc/pin_bloc.dart';
import 'feature/pin/pin_overlay.dart';
import 'feature/pin/pin_prompt.dart';
import 'feature/pin/pin_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';
import 'feature/theme/theme_screen.dart';
import 'feature/verification/bloc/verification_bloc.dart';
import 'feature/verification/verification_screen.dart';
import 'feature/verifier_policy/bloc/verifier_policy_bloc.dart';
import 'feature/verifier_policy/verifier_policy_screen.dart';
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
  static const confirmRoute = '/confirm';
  static const walletCreateRoute = '/wallet/create';
  static const homeRoute = '/home';
  static const cardAddRoute = '/card/add';
  static const cardSummaryRoute = '/card/summary';
  static const cardDataRoute = '/card/data';
  static const cardHistoryRoute = '/card/history';
  static const themeRoute = '/theme';
  static const verificationRoute = '/verification';
  static const issuanceRoute = '/issuance';
  static const verifierPolicyRoute = '/verifier/policy';

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
        return _createSplashScreenBuilder;
      case WalletRoutes.pinRoute:
        return _createPinScreenBuilder;
      case WalletRoutes.confirmRoute:
        return _createConfirmScreenBuilder;
      case WalletRoutes.homeRoute:
        return _createHomeScreenBuilder;
      case WalletRoutes.cardAddRoute:
        return _createCardAddScreenBuilder;
      case WalletRoutes.cardSummaryRoute:
        return _createCardSummaryScreenBuilder(settings);
      case WalletRoutes.cardDataRoute:
        return _createCardDataScreenBuilder(settings);
      case WalletRoutes.cardHistoryRoute:
        return _createCardHistoryScreenBuilder(settings);
      case WalletRoutes.themeRoute:
        return _createThemeScreenBuilder;
      case WalletRoutes.verificationRoute:
        return _createVerificationScreenBuilder(settings);
      case WalletRoutes.verifierPolicyRoute:
        return _createVerifierPolicyScreenBuilder(settings);
      case WalletRoutes.issuanceRoute:
        return _createIssuanceScreenBuilder(settings);
      case WalletRoutes.walletCreateRoute:
        return _createWalletCreateScreenBuilder;
      default:
        throw UnsupportedError('Unknown route: ${settings.name}');
    }
  }

  static List<Route<dynamic>> initialRoutes(String route) => [MaterialPageRoute(builder: _createSplashScreenBuilder)];
}

Widget _createSplashScreenBuilder(BuildContext context) => BlocProvider<SplashBloc>(
      create: (BuildContext context) => SplashBloc(context.read()),
      child: const SplashScreen(),
    );

Widget _createPinScreenBuilder(BuildContext context) => BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<UnlockWalletUseCase>(), context.read()),
      child: PinScreen(onUnlock: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute)),
    );

Widget _createConfirmScreenBuilder(BuildContext context) => const PinPrompt();

Widget _createHomeScreenBuilder(BuildContext context) => BlocProvider<HomeBloc>(
      create: (BuildContext context) => HomeBloc(),
      child: const HomeScreen(),
    );

Widget _createCardAddScreenBuilder(BuildContext context) => const CardAddScreen();

WidgetBuilder _createCardSummaryScreenBuilder(RouteSettings settings) {
  return (context) {
    final String cardId = CardSummaryScreen.getArguments(settings);
    return BlocProvider<CardSummaryBloc>(
      create: (context) => CardSummaryBloc(context.read())..add(CardSummaryLoadTriggered(cardId)),
      child: const CardSummaryScreen(),
    );
  };
}

WidgetBuilder _createCardDataScreenBuilder(RouteSettings settings) {
  return (context) {
    final String cardId = CardDataScreen.getArguments(settings);
    return BlocProvider<CardDataBloc>(
      create: (context) => CardDataBloc(context.read())..add(CardDataLoadTriggered(cardId)),
      child: const CardDataScreen(),
    );
  };
}

WidgetBuilder _createCardHistoryScreenBuilder(RouteSettings settings) {
  return (context) {
    final String cardId = CardHistoryScreen.getArguments(settings);
    return BlocProvider<CardHistoryBloc>(
      create: (context) => CardHistoryBloc(context.read())..add(CardHistoryLoadTriggered(cardId)),
      child: const CardHistoryScreen(),
    );
  };
}

Widget _createThemeScreenBuilder(BuildContext context) => const ThemeScreen();

WidgetBuilder _createVerificationScreenBuilder(RouteSettings settings) {
  String sessionId = VerificationScreen.getArguments(settings);
  return (context) {
    return BlocProvider<VerificationBloc>(
      create: (BuildContext context) => VerificationBloc(context.read())..add(VerificationLoadRequested(sessionId)),
      child: const VerificationScreen(),
    );
  };
}

WidgetBuilder _createVerifierPolicyScreenBuilder(RouteSettings settings) {
  return (context) {
    String sessionId = VerifierPolicyScreen.getArguments(settings);
    return BlocProvider<VerifierPolicyBloc>(
      create: (BuildContext context) => VerifierPolicyBloc(context.read())..add(VerifierPolicyLoadTriggered(sessionId)),
      child: const VerifierPolicyScreen(),
    );
  };
}

WidgetBuilder _createIssuanceScreenBuilder(RouteSettings settings) {
  return (context) {
    String sessionId = IssuanceScreen.getArguments(settings);
    return BlocProvider<IssuanceBloc>(
      create: (BuildContext context) => IssuanceBloc(context.read())..add(IssuanceLoadTriggered(sessionId)),
      child: const IssuanceScreen(),
    );
  };
}

Widget _createWalletCreateScreenBuilder(BuildContext context) => const WalletCreateScreen();

class SecuredPageRoute<T> extends MaterialPageRoute<T> {
  SecuredPageRoute({required WidgetBuilder builder, super.settings})
      : super(builder: (context) => PinOverlay(child: builder(context)));
}
