import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'feature/card/add/card_add_screen.dart';
import 'feature/card/data/bloc/card_data_bloc.dart';
import 'feature/card/data/card_data_screen.dart';
import 'feature/card/history/bloc/card_history_bloc.dart';
import 'feature/card/history/card_history_screen.dart';
import 'feature/card/overview/bloc/card_overview_bloc.dart';
import 'feature/card/share/card_share_screen.dart';
import 'feature/card/summary/bloc/card_summary_bloc.dart';
import 'feature/card/summary/card_summary_screen.dart';
import 'feature/history/detail/bloc/history_detail_bloc.dart';
import 'feature/history/detail/history_detail_screen.dart';
import 'feature/history/overview/bloc/history_overview_bloc.dart';
import 'feature/history/overview/history_overview_screen.dart';
import 'feature/home/bloc/home_bloc.dart';
import 'feature/home/home_screen.dart';
import 'feature/introduction/introduction_screen.dart';
import 'feature/issuance/argument/issuance_screen_argument.dart';
import 'feature/issuance/bloc/issuance_bloc.dart';
import 'feature/issuance/issuance_screen.dart';
import 'feature/menu/bloc/menu_bloc.dart';
import 'feature/pin/bloc/pin_bloc.dart';
import 'feature/pin/pin_overlay.dart';
import 'feature/pin/pin_prompt.dart';
import 'feature/pin/pin_screen.dart';
import 'feature/setup_security/bloc/setup_security_bloc.dart';
import 'feature/setup_security/setup_security_screen.dart';
import 'feature/sign/bloc/sign_bloc.dart';
import 'feature/sign/sign_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';
import 'feature/theme/theme_screen.dart';
import 'feature/verification/bloc/verification_bloc.dart';
import 'feature/verification/verification_screen.dart';
import 'feature/verifier_policy/bloc/verifier_policy_bloc.dart';
import 'feature/verifier_policy/verifier_policy_screen.dart';
import 'feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'feature/wallet/personalize/wallet_personalize_screen.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  /// Routes in this list will be shown WITHOUT pin (wallet unlock) requirement
  @visibleForTesting
  static const publicRoutes = [splashRoute, introductionRoute, setupSecurityRoute, pinRoute, themeRoute];

  static const splashRoute = '/';
  static const introductionRoute = '/introduction';
  static const setupSecurityRoute = '/security/setup';
  static const pinRoute = '/pin';
  static const confirmRoute = '/confirm';
  static const walletPersonalizeRoute = '/wallet/personalize';
  static const walletHistoryRoute = '/wallet/history';
  static const homeRoute = '/home';
  static const cardAddRoute = '/card/add';
  static const cardSummaryRoute = '/card/summary';
  static const cardDataRoute = '/card/data';
  static const cardHistoryRoute = '/card/history';
  static const cardShareRoute = '/card/share';
  static const themeRoute = '/theme';
  static const verificationRoute = '/verification';
  static const issuanceRoute = '/issuance';
  static const signRoute = '/sign';
  static const verifierPolicyRoute = '/verifier/policy';
  static const historyDetailRoute = '/history';

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
      case WalletRoutes.introductionRoute:
        return _createIntroductionScreenBuilder;
      case WalletRoutes.pinRoute:
        return _createPinScreenBuilder;
      case WalletRoutes.setupSecurityRoute:
        return _createSetupSecurityScreenBuilder;
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
      case WalletRoutes.cardShareRoute:
        return _createCardShareScreenBuilder(settings);
      case WalletRoutes.themeRoute:
        return _createThemeScreenBuilder;
      case WalletRoutes.verificationRoute:
        return _createVerificationScreenBuilder(settings);
      case WalletRoutes.verifierPolicyRoute:
        return _createVerifierPolicyScreenBuilder(settings);
      case WalletRoutes.issuanceRoute:
        return _createIssuanceScreenBuilder(settings);
      case WalletRoutes.signRoute:
        return _createSignScreenBuilder(settings);
      case WalletRoutes.walletPersonalizeRoute:
        return _createWalletPersonalizeScreenBuilder;
      case WalletRoutes.walletHistoryRoute:
        return _createHistoryOverviewScreenBuilder;
      case WalletRoutes.historyDetailRoute:
        return _createHistoryDetailScreenBuilder(settings);
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

Widget _createIntroductionScreenBuilder(BuildContext context) => const IntroductionScreen();

Widget _createConfirmScreenBuilder(BuildContext context) => const PinPrompt();

Widget _createPinScreenBuilder(BuildContext context) => BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<UnlockWalletWithPinUseCase>(), context.read()),
      child: PinScreen(onUnlock: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute)),
    );

Widget _createSetupSecurityScreenBuilder(BuildContext context) => BlocProvider<SetupSecurityBloc>(
      create: (BuildContext context) => SetupSecurityBloc(context.read(), context.read(), context.read()),
      child: const SetupSecurityScreen(),
    );

Widget _createHomeScreenBuilder(BuildContext context) => MultiBlocProvider(
      providers: [
        BlocProvider<HomeBloc>(
          create: (BuildContext context) => HomeBloc(),
        ),
        BlocProvider<CardOverviewBloc>(
          create: (BuildContext context) => CardOverviewBloc(context.read(), context.read()),
        ),
        BlocProvider<MenuBloc>(
          create: (BuildContext context) => MenuBloc(context.read(), context.read()),
        ),
      ],
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
      create: (context) => CardHistoryBloc(context.read(), context.read())..add(CardHistoryLoadTriggered(cardId)),
      child: const CardHistoryScreen(),
    );
  };
}

WidgetBuilder _createCardShareScreenBuilder(RouteSettings settings) {
  return (context) {
    final String screenTitle = CardShareScreen.getArguments(settings);
    return CardShareScreen(screenTitle: screenTitle);
  };
}

Widget _createThemeScreenBuilder(BuildContext context) => const ThemeScreen();

WidgetBuilder _createVerificationScreenBuilder(RouteSettings settings) {
  String sessionId = VerificationScreen.getArguments(settings);
  return (context) {
    return BlocProvider<VerificationBloc>(
      create: (BuildContext context) {
        return VerificationBloc(context.read(), context.read(), context.read())
          ..add(VerificationLoadRequested(sessionId));
      },
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
    IssuanceScreenArgument argument = IssuanceScreen.getArgument(settings);
    return BlocProvider<IssuanceBloc>(
      create: (BuildContext context) {
        return IssuanceBloc(context.read(), context.read(), context.read(), context.read())
          ..add(IssuanceLoadTriggered(argument.sessionId, argument.isRefreshFlow));
      },
      child: const IssuanceScreen(),
    );
  };
}

WidgetBuilder _createSignScreenBuilder(RouteSettings settings) {
  return (context) {
    String sessionId = SignScreen.getArguments(settings);
    return BlocProvider<SignBloc>(
      create: (BuildContext context) {
        return SignBloc(context.read(), context.read(), context.read())..add(SignLoadTriggered(sessionId));
      },
      child: const SignScreen(),
    );
  };
}

Widget _createWalletPersonalizeScreenBuilder(BuildContext context) {
  return BlocProvider<WalletPersonalizeBloc>(
    create: (BuildContext context) => WalletPersonalizeBloc(context.read(), context.read()),
    child: const WalletPersonalizeScreen(),
  );
}

Widget _createHistoryOverviewScreenBuilder(BuildContext context) {
  return BlocProvider<HistoryOverviewBloc>(
    create: (BuildContext context) => HistoryOverviewBloc(context.read()),
    child: const HistoryOverviewScreen(),
  );
}

WidgetBuilder _createHistoryDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    String attributeId = HistoryDetailScreen.getArguments(settings);
    return BlocProvider<HistoryDetailBloc>(
      create: (BuildContext context) => HistoryDetailBloc(context.read())..add(HistoryDetailLoadTriggered(attributeId)),
      child: const HistoryDetailScreen(),
    );
  };
}

class SecuredPageRoute<T> extends MaterialPageRoute<T> {
  SecuredPageRoute({required WidgetBuilder builder, super.settings})
      : super(builder: (context) => PinOverlay(child: builder(context)));
}
