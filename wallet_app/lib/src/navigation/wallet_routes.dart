import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/service/navigation_service.dart';
import '../domain/model/attribute/attribute.dart';
import '../domain/model/policy/policy.dart';
import '../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../feature/about/about_screen.dart';
import '../feature/card/data/argument/card_data_screen_argument.dart';
import '../feature/card/data/bloc/card_data_bloc.dart';
import '../feature/card/data/card_data_screen.dart';
import '../feature/card/detail/argument/card_detail_screen_argument.dart';
import '../feature/card/detail/bloc/card_detail_bloc.dart';
import '../feature/card/detail/card_detail_screen.dart';
import '../feature/card/history/bloc/card_history_bloc.dart';
import '../feature/card/history/card_history_screen.dart';
import '../feature/change_language/bloc/change_language_bloc.dart';
import '../feature/change_language/change_language_screen.dart';
import '../feature/common/widget/utility/do_on_init.dart';
import '../feature/dashboard/bloc/dashboard_bloc.dart';
import '../feature/disclosure/bloc/disclosure_bloc.dart';
import '../feature/disclosure/disclosure_screen.dart';
import '../feature/history/detail/argument/history_detail_screen_argument.dart';
import '../feature/history/detail/bloc/history_detail_bloc.dart';
import '../feature/history/detail/history_detail_screen.dart';
import '../feature/history/overview/bloc/history_overview_bloc.dart';
import '../feature/history/overview/history_overview_screen.dart';
import '../feature/home/argument/home_screen_argument.dart';
import '../feature/home/bloc/home_bloc.dart';
import '../feature/home/home_screen.dart';
import '../feature/introduction/introduction_conditions_screen.dart';
import '../feature/introduction/introduction_expectations_screen.dart';
import '../feature/introduction/introduction_privacy_screen.dart';
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
import '../feature/qr/bloc/qr_bloc.dart';
import '../feature/qr/qr_screen.dart';
import '../feature/settings/settings_screen.dart';
import '../feature/setup_security/bloc/setup_security_bloc.dart';
import '../feature/setup_security/setup_security_screen.dart';
import '../feature/sign/bloc/sign_bloc.dart';
import '../feature/sign/sign_screen.dart';
import '../feature/splash/bloc/splash_bloc.dart';
import '../feature/splash/splash_screen.dart';
import '../feature/theme/theme_screen.dart';
import '../feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import '../feature/wallet/personalize/wallet_personalize_screen.dart';
import '../util/cast_util.dart';
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
    introductionExpectationsRoute,
    introductionPrivacyRoute,
    introductionConditionsRoute,
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
  static const introductionExpectationsRoute = '/introduction/expectations';
  static const introductionPrivacyRoute = '/introduction/privacy';
  static const introductionConditionsRoute = '/introduction/conditions';
  static const aboutRoute = '/about';
  static const setupSecurityRoute = '/security/setup';
  static const pinRoute = '/pin';
  static const pinTimeoutRoute = '/pin/timeout';
  static const pinBlockedRoute = '/pin/blocked';
  static const confirmRoute = '/confirm';
  static const walletPersonalizeRoute = '/wallet/personalize';
  static const walletHistoryRoute = '/wallet/history';
  static const homeRoute = '/home';
  static const cardDetailRoute = '/card/detail';
  static const cardDataRoute = '/card/data';
  static const cardHistoryRoute = '/card/history';
  static const themeRoute = '/theme';
  static const disclosureRoute = '/disclosure';
  static const issuanceRoute = '/issuance';
  static const signRoute = '/sign';
  static const policyRoute = '/policy';
  static const historyDetailRoute = '/history';
  static const changeLanguageRoute = '/language';
  static const organizationDetailRoute = '/organization';
  static const settingsRoute = '/settings';
  static const qrRoute = '/qr';

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
      case WalletRoutes.disclosureRoute:
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
      case WalletRoutes.qrRoute:
        return _createQrScreenBuilder;
      case WalletRoutes.introductionRoute:
        return _createIntroductionScreenBuilder;
      case WalletRoutes.introductionExpectationsRoute:
        return _createIntroductionExpectationsScreenBuilder;
      case WalletRoutes.introductionPrivacyRoute:
        return _createIntroductionPrivacyScreenBuilder;
      case WalletRoutes.introductionConditionsRoute:
        return _createIntroductionConditionsScreenBuilder;
      case WalletRoutes.aboutRoute:
        return _createAboutScreenBuilder;
      case WalletRoutes.pinRoute:
        return _createPinScreenBuilder;
      case WalletRoutes.setupSecurityRoute:
        return _createSetupSecurityScreenBuilder;
      case WalletRoutes.confirmRoute:
        return _createConfirmScreenBuilder;
      case WalletRoutes.homeRoute:
        return _createHomeScreenBuilder(settings);
      case WalletRoutes.cardDetailRoute:
        return _createCardDetailScreenBuilder(settings);
      case WalletRoutes.cardDataRoute:
        return _createCardDataScreenBuilder(settings);
      case WalletRoutes.cardHistoryRoute:
        return _createCardHistoryScreenBuilder(settings);
      case WalletRoutes.themeRoute:
        return _createThemeScreenBuilder;
      case WalletRoutes.disclosureRoute:
        return _createDisclosureScreenBuilder(settings);
      case WalletRoutes.policyRoute:
        return _createPolicyScreenBuilder(settings);
      case WalletRoutes.issuanceRoute:
        return _createIssuanceScreenBuilder(settings);
      case WalletRoutes.signRoute:
        return _createSignScreenBuilder(settings);
      case WalletRoutes.walletPersonalizeRoute:
        return _createWalletPersonalizeScreenBuilder(settings);
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

Widget _createQrScreenBuilder(BuildContext context) => BlocProvider<QrBloc>(
      create: (BuildContext context) => QrBloc(context.read()),
      child: const QrScreen(),
    );

Widget _createIntroductionScreenBuilder(BuildContext context) => const IntroductionScreen();

Widget _createIntroductionExpectationsScreenBuilder(BuildContext context) => const IntroductionExpectationsScreen();

Widget _createIntroductionPrivacyScreenBuilder(BuildContext context) => const IntroductionPrivacyScreen();

Widget _createIntroductionConditionsScreenBuilder(BuildContext context) => const IntroductionConditionsScreen();

Widget _createAboutScreenBuilder(BuildContext context) => const AboutScreen();

Widget _createConfirmScreenBuilder(BuildContext context) => const PinPrompt();

Widget _createPinScreenBuilder(BuildContext context) => BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<UnlockWalletWithPinUseCase>()),
      child: PinScreen(onUnlock: (_) => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute)),
    );

Widget _createSetupSecurityScreenBuilder(BuildContext context) => BlocProvider<SetupSecurityBloc>(
      create: (BuildContext context) => SetupSecurityBloc(context.read(), context.read(), context.read()),
      child: const SetupSecurityScreen(),
    );

WidgetBuilder _createHomeScreenBuilder(RouteSettings settings) {
  final HomeScreenArgument? argument = HomeScreen.getArgument(settings);
  return (context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<HomeBloc>(
          create: (BuildContext context) => HomeBloc(),
        ),
        BlocProvider<DashboardBloc>(
          create: (BuildContext context) => DashboardBloc(
            context.read(),
            argument?.cards,
          ),
        ),
        BlocProvider<MenuBloc>(
          create: (BuildContext context) => MenuBloc(context.read(), context.read()),
        ),
      ],
      child: DoOnInit(
        child: const HomeScreen(),
        onInit: (context) => context.read<NavigationService>().processQueue(),
      ),
    );
  };
}

WidgetBuilder _createCardDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    CardDetailScreenArgument argument = CardDetailScreen.getArgument(settings);
    return BlocProvider<CardDetailBloc>(
      create: (context) => CardDetailBloc(context.read(), argument.card)..add(CardDetailLoadTriggered(argument.cardId)),
      child: CardDetailScreen(cardTitle: argument.cardTitle.l10nValue(context)),
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
    final String docType = CardHistoryScreen.getArguments(settings);
    return BlocProvider<CardHistoryBloc>(
      create: (context) => CardHistoryBloc(context.read(), context.read())..add(CardHistoryLoadTriggered(docType)),
      child: const CardHistoryScreen(),
    );
  };
}

Widget _createThemeScreenBuilder(BuildContext context) => const ThemeScreen();

WidgetBuilder _createDisclosureScreenBuilder(RouteSettings settings) {
  final args = DisclosureScreen.getArgument(settings);
  return (context) {
    return BlocProvider<DisclosureBloc>(
      create: (BuildContext context) => DisclosureBloc(
        context.read(),
        context.read(),
      )..add(DisclosureSessionStarted(args.uri!)),
      child: const DisclosureScreen(),
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
        return IssuanceBloc(
          argument.uri!,
          argument.isRefreshFlow,
          context.read(),
          context.read(),
          context.read(),
          context.read(),
        );
      },
      child: const IssuanceScreen(),
    );
  };
}

WidgetBuilder _createSignScreenBuilder(RouteSettings settings) {
  return (context) {
    final arguments = SignScreen.getArgument(settings);
    return BlocProvider<SignBloc>(
      create: (BuildContext context) {
        return SignBloc(
          arguments.uri!,
          context.read(),
          context.read(),
        );
      },
      child: const SignScreen(),
    );
  };
}

WidgetBuilder _createWalletPersonalizeScreenBuilder(RouteSettings settings) {
  return (context) {
    String? argument = tryCast<String>(settings.arguments);

    return BlocProvider<WalletPersonalizeBloc>(
      create: (BuildContext context) => WalletPersonalizeBloc(
        argument,
        context.read(),
        context.read(),
        context.read(),
        context.read(),
        context.read(),
      ),
      child: const WalletPersonalizeScreen(),
    );
  };
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
      create: (BuildContext context) => HistoryDetailBloc(context.read())
        ..add(HistoryDetailLoadTriggered(
          attribute: argument.timelineAttribute,
          docType: argument.docType,
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
      create: (BuildContext context) => OrganizationDetailBloc()
        ..add(
          OrganizationProvided(
            organization: argument.organization,
            sharedDataWithOrganizationBefore: argument.sharedDataWithOrganizationBefore,
          ),
        ),
      child: const OrganizationDetailScreen(),
    );
  };
}

Widget _createSettingsScreenBuilder(BuildContext context) => const SettingsScreen();
