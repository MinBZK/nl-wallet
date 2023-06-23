import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/service/deeplink_service.dart';
import '../domain/model/policy/policy.dart';
import '../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../feature/about/about_screen.dart';
import '../feature/card/data/argument/card_data_screen_argument.dart';
import '../feature/card/data/bloc/card_data_bloc.dart';
import '../feature/card/data/card_data_screen.dart';
import '../feature/card/history/bloc/card_history_bloc.dart';
import '../feature/card/history/card_history_screen.dart';
import '../feature/card/overview/bloc/card_overview_bloc.dart';
import '../feature/card/summary/argument/card_summary_screen_argument.dart';
import '../feature/card/summary/bloc/card_summary_bloc.dart';
import '../feature/card/summary/card_summary_screen.dart';
import '../feature/change_language/bloc/change_language_bloc.dart';
import '../feature/change_language/change_language_screen.dart';
import '../feature/common/widget/utility/do_on_init.dart';
import '../feature/history/detail/argument/history_detail_screen_argument.dart';
import '../feature/history/detail/bloc/history_detail_bloc.dart';
import '../feature/history/detail/history_detail_screen.dart';
import '../feature/history/overview/bloc/history_overview_bloc.dart';
import '../feature/history/overview/history_overview_screen.dart';
import '../feature/home/bloc/home_bloc.dart';
import '../feature/home/home_screen.dart';
import '../feature/introduction/introduction_screen.dart';
import '../feature/issuance/argument/issuance_screen_argument.dart';
import '../feature/issuance/bloc/issuance_bloc.dart';
import '../feature/issuance/issuance_screen.dart';
import '../feature/menu/bloc/menu_bloc.dart';
import '../feature/organization/detail/argument/organization_detail_screen_argument.dart';
import '../feature/organization/detail/bloc/organization_detail_bloc.dart';
import '../feature/organization/detail/organization_detail_screen.dart';
import '../feature/pin/bloc/pin_bloc.dart';
import '../feature/pin/pin_prompt.dart';
import '../feature/pin/pin_screen.dart';
import '../feature/pin_blocked/pin_blocked_screen.dart';
import '../feature/pin_timeout/pin_timeout_screen.dart';
import '../feature/policy/policy_screen.dart';
import '../feature/settings/settings_screen.dart';
import '../feature/setup_security/bloc/setup_security_bloc.dart';
import '../feature/setup_security/setup_security_screen.dart';
import '../feature/sign/bloc/sign_bloc.dart';
import '../feature/sign/sign_screen.dart';
import '../feature/splash/bloc/splash_bloc.dart';
import '../feature/splash/splash_screen.dart';
import '../feature/theme/theme_screen.dart';
import '../feature/verification/bloc/verification_bloc.dart';
import '../feature/verification/verification_screen.dart';
import '../feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import '../feature/wallet/personalize/wallet_personalize_screen.dart';
import 'secured_page_route.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  /// Routes in this list will be shown WITHOUT pin (wallet unlock) requirement
  @visibleForTesting
  static const publicRoutes = [
    splashRoute,
    introductionRoute,
    aboutRoute,
    setupSecurityRoute,
    pinRoute,
    pinTimeoutRoute,
    pinBlockedRoute,
    themeRoute,
    changeLanguageRoute,
  ];

  static const splashRoute = '/';
  static const introductionRoute = '/introduction';
  static const aboutRoute = '/about';
  static const setupSecurityRoute = '/security/setup';
  static const pinRoute = '/pin';
  static const pinTimeoutRoute = '/pin/timeout';
  static const pinBlockedRoute = '/pin/blocked';
  static const confirmRoute = '/confirm';
  static const walletPersonalizeRoute = '/wallet/personalize';
  static const walletHistoryRoute = '/wallet/history';
  static const homeRoute = '/home';
  static const cardSummaryRoute = '/card/summary';
  static const cardDataRoute = '/card/data';
  static const cardHistoryRoute = '/card/history';
  static const themeRoute = '/theme';
  static const verificationRoute = '/verification';
  static const issuanceRoute = '/issuance';
  static const signRoute = '/sign';
  static const policyRoute = '/policy';
  static const historyDetailRoute = '/history';
  static const changeLanguageRoute = '/language';
  static const organizationDetailRoute = '/organization';
  static const settingsRoute = '/settings';

  static Route<dynamic> routeFactory(RouteSettings settings) {
    WidgetBuilder builder = _widgetBuilderFactory(settings);
    final pageTransition = _resolvePageTransition(settings);
    if (publicRoutes.contains(settings.name)) {
      return MaterialPageRoute(builder: builder, settings: settings);
    } else {
      return SecuredPageRoute(builder: builder, settings: settings, transition: pageTransition);
    }
  }

  static SecuredPageTransition _resolvePageTransition(RouteSettings settings) {
    switch (settings.name) {
      case WalletRoutes.verificationRoute:
      case WalletRoutes.issuanceRoute:
      case WalletRoutes.signRoute:
        return SecuredPageTransition.slideInFromBottom;
      default:
        return SecuredPageTransition.platform;
    }
  }

  static WidgetBuilder _widgetBuilderFactory(RouteSettings settings) {
    switch (settings.name) {
      case WalletRoutes.splashRoute:
        return _createSplashScreenBuilder;
      case WalletRoutes.introductionRoute:
        return _createIntroductionScreenBuilder;
      case WalletRoutes.aboutRoute:
        return _createAboutScreenBuilder;
      case WalletRoutes.pinRoute:
        return _createPinScreenBuilder;
      case WalletRoutes.setupSecurityRoute:
        return _createSetupSecurityScreenBuilder;
      case WalletRoutes.confirmRoute:
        return _createConfirmScreenBuilder;
      case WalletRoutes.homeRoute:
        return _createHomeScreenBuilder;
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
      case WalletRoutes.policyRoute:
        return _createPolicyScreenBuilder(settings);
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
      case WalletRoutes.organizationDetailRoute:
        return _createOrganizationDetailScreenBuilder(settings);
      case WalletRoutes.changeLanguageRoute:
        return _createChangeLanguageScreenBuilder;
      case WalletRoutes.pinTimeoutRoute:
        return _createPinTimeoutScreenBuilder(settings);
      case WalletRoutes.pinBlockedRoute:
        return _createPinBlockedScreenBuilder(settings);
      case WalletRoutes.settingsRoute:
        return _createSettingsScreenBuilder;
      default:
        throw UnsupportedError('Unknown route: ${settings.name}');
    }
  }

  static List<Route<dynamic>> initialRoutes(String route) => [MaterialPageRoute(builder: _createSplashScreenBuilder)];
}

Widget _createSplashScreenBuilder(BuildContext context) => BlocProvider<SplashBloc>(
      create: (BuildContext context) => SplashBloc(context.read(), context.read()),
      child: const SplashScreen(),
    );

Widget _createIntroductionScreenBuilder(BuildContext context) => const IntroductionScreen();

Widget _createAboutScreenBuilder(BuildContext context) => const AboutScreen();

Widget _createConfirmScreenBuilder(BuildContext context) => const PinPrompt();

Widget _createPinScreenBuilder(BuildContext context) => BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<UnlockWalletWithPinUseCase>()),
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
      child: DoOnInit(
        child: const HomeScreen(),
        onInit: (context) => context.read<DeeplinkService>().processQueue(),
      ),
    );

WidgetBuilder _createCardSummaryScreenBuilder(RouteSettings settings) {
  return (context) {
    CardSummaryScreenArgument argument = CardSummaryScreen.getArgument(settings);
    return BlocProvider<CardSummaryBloc>(
      create: (context) => CardSummaryBloc(context.read())..add(CardSummaryLoadTriggered(argument.cardId)),
      child: CardSummaryScreen(cardTitle: argument.cardTitle),
    );
  };
}

WidgetBuilder _createCardDataScreenBuilder(RouteSettings settings) {
  return (context) {
    final CardDataScreenArgument argument = CardDataScreen.getArgument(settings);
    return BlocProvider<CardDataBloc>(
      create: (context) => CardDataBloc(context.read())..add(CardDataLoadTriggered(argument.cardId)),
      child: CardDataScreen(cardTitle: argument.cardTitle),
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

Widget _createThemeScreenBuilder(BuildContext context) => const ThemeScreen();

WidgetBuilder _createVerificationScreenBuilder(RouteSettings settings) {
  String sessionId = VerificationScreen.getArguments(settings);
  return (context) {
    return BlocProvider<VerificationBloc>(
      create: (BuildContext context) {
        return VerificationBloc(context.read(), context.read(), context.read(), context.read())
          ..add(VerificationLoadRequested(sessionId));
      },
      child: const VerificationScreen(),
    );
  };
}

WidgetBuilder _createPolicyScreenBuilder(RouteSettings settings) {
  return (context) {
    Policy policy = PolicyScreen.getArguments(settings);
    return PolicyScreen(policy: policy);
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
    create: (BuildContext context) => WalletPersonalizeBloc(
      context.read(),
      context.read(),
      context.read(),
      context.read(),
      context.read(),
    ),
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
    HistoryDetailScreenArgument argument = HistoryDetailScreen.getArgument(settings);
    return BlocProvider<HistoryDetailBloc>(
      create: (BuildContext context) => HistoryDetailBloc(context.read(), context.read())
        ..add(HistoryDetailLoadTriggered(
          attributeId: argument.timelineAttributeId,
          cardId: argument.cardId,
        )),
      child: const HistoryDetailScreen(),
    );
  };
}

Widget _createChangeLanguageScreenBuilder(BuildContext context) {
  return BlocProvider<ChangeLanguageBloc>(
    create: (BuildContext context) => ChangeLanguageBloc(context.read(), () => Localizations.localeOf(context)),
    child: const ChangeLanguageScreen(),
  );
}

WidgetBuilder _createPinTimeoutScreenBuilder(RouteSettings settings) {
  return (context) {
    final arguments = PinTimeoutScreen.getArgument(settings);
    return PinTimeoutScreen(expiryTime: arguments.expiryTime);
  };
}

WidgetBuilder _createPinBlockedScreenBuilder(RouteSettings settings) {
  return (context) => const PinBlockedScreen();
}

WidgetBuilder _createOrganizationDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    OrganizationDetailScreenArgument argument = OrganizationDetailScreen.getArgument(settings);
    return BlocProvider<OrganizationDetailBloc>(
      create: (BuildContext context) => OrganizationDetailBloc(context.read(), context.read())
        ..add(
          OrganizationLoadTriggered(organizationId: argument.organizationId),
        ),
      child: const OrganizationDetailScreen(),
    );
  };
}

Widget _createSettingsScreenBuilder(BuildContext context) => const SettingsScreen();
