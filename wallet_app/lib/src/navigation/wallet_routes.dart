import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../domain/model/attribute/attribute.dart';
import '../domain/model/consumable.dart';
import '../domain/model/result/application_error.dart';
import '../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../domain/usecase/transfer/confirm_wallet_transfer_usecase.dart';
import '../feature/about/about_screen.dart';
import '../feature/biometric_settings/biometric_settings_screen.dart';
import '../feature/biometric_settings/bloc/biometric_settings_bloc.dart';
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
import '../feature/change_pin/bloc/change_pin_bloc.dart';
import '../feature/change_pin/change_pin_screen.dart';
import '../feature/dashboard/argument/dashboard_screen_argument.dart';
import '../feature/dashboard/bloc/dashboard_bloc.dart';
import '../feature/dashboard/dashboard_screen.dart';
import '../feature/demo/demo_screen.dart';
import '../feature/disclosure/bloc/disclosure_bloc.dart';
import '../feature/disclosure/disclosure_screen.dart';
import '../feature/forgot_pin/forgot_pin_screen.dart';
import '../feature/history/detail/argument/history_detail_screen_argument.dart';
import '../feature/history/detail/bloc/history_detail_bloc.dart';
import '../feature/history/detail/history_detail_screen.dart';
import '../feature/history/overview/bloc/history_overview_bloc.dart';
import '../feature/history/overview/history_overview_screen.dart';
import '../feature/introduction/introduction_privacy_screen.dart';
import '../feature/introduction/introduction_screen.dart';
import '../feature/issuance/argument/issuance_screen_argument.dart';
import '../feature/issuance/bloc/issuance_bloc.dart';
import '../feature/issuance/issuance_screen.dart';
import '../feature/login/login_detail_screen.dart';
import '../feature/menu/bloc/menu_bloc.dart';
import '../feature/menu/menu_screen.dart';
import '../feature/menu/sub_menu/contact/contact_screen.dart';
import '../feature/menu/sub_menu/need_help/need_help_screen.dart';
import '../feature/menu/sub_menu/settings/settings_screen.dart';
import '../feature/notification/bloc/manage_notifications_bloc.dart';
import '../feature/notification/manage_notifications_screen.dart';
import '../feature/organization/detail/argument/organization_detail_screen_argument.dart';
import '../feature/organization/detail/bloc/organization_detail_bloc.dart';
import '../feature/organization/detail/organization_detail_screen.dart';
import '../feature/pin/bloc/pin_bloc.dart';
import '../feature/pin/pin_screen.dart';
import '../feature/pin_blocked/pin_blocked_screen.dart';
import '../feature/pin_timeout/pin_timeout_screen.dart';
import '../feature/policy/policy_screen.dart';
import '../feature/policy/policy_screen_arguments.dart';
import '../feature/privacy_policy/privacy_policy_screen.dart';
import '../feature/qr/bloc/qr_bloc.dart';
import '../feature/qr/qr_screen.dart';
import '../feature/recover_pin/bloc/recover_pin_bloc.dart';
import '../feature/recover_pin/recover_pin_screen.dart';
import '../feature/renew_pid/bloc/renew_pid_bloc.dart';
import '../feature/renew_pid/renew_pid_screen.dart';
import '../feature/review_revocation_code_screen/bloc/review_revocation_code_bloc.dart';
import '../feature/review_revocation_code_screen/review_revocation_code_screen.dart';
import '../feature/revocation/bloc/revocation_code_bloc.dart';
import '../feature/revocation/revocation_code_screen.dart';
import '../feature/setup_security/bloc/setup_security_bloc.dart';
import '../feature/setup_security/setup_security_screen.dart';
import '../feature/sign/bloc/sign_bloc.dart';
import '../feature/sign/sign_screen.dart';
import '../feature/splash/bloc/splash_bloc.dart';
import '../feature/splash/splash_screen.dart';
import '../feature/theme/theme_screen.dart';
import '../feature/tour/overview/bloc/tour_overview_bloc.dart';
import '../feature/tour/overview/tour_overview_screen.dart';
import '../feature/tour/video/argument/tour_video_screen_argument.dart';
import '../feature/tour/video/tour_video_screen.dart';
import '../feature/update/update_info_screen.dart';
import '../feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import '../feature/wallet/personalize/wallet_personalize_screen.dart';
import '../feature/wallet_transfer_faq/wallet_transfer_faq_screen.dart';
import '../feature/wallet_transfer_source/bloc/wallet_transfer_source_bloc.dart';
import '../feature/wallet_transfer_source/wallet_transfer_source_screen.dart';
import '../feature/wallet_transfer_target/bloc/wallet_transfer_target_bloc.dart'
    hide WalletTransferGenericError, WalletTransferUpdateStateEvent;
import '../feature/wallet_transfer_target/wallet_transfer_target_screen.dart';
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
    introductionPrivacyRoute,
    privacyPolicyRoute,
    aboutRoute,
    pinRoute,
    pinTimeoutRoute,
    pinRecoveryRoute,
    pinBlockedRoute,
    forgotPinRoute,
    updateInfoRoute,
  ];

  static const aboutRoute = '/about';
  static const biometricsSettingsRoute = '/settings/biometrics';
  static const cardDataRoute = '/card/data';
  static const cardDetailRoute = '/card/detail';
  static const cardHistoryRoute = '/card/history';
  static const changeLanguageRoute = '/language';
  static const changePinRoute = '/change_pin';
  static const contactRoute = '/menu/contact';
  static const dashboardRoute = '/dashboard';
  static const demoRoute = '/demo';
  static const disclosureRoute = '/disclosure';
  static const forgotPinRoute = '/forgot_pin';
  static const historyDetailRoute = '/history';
  static const introductionPrivacyRoute = '/introduction/privacy';
  static const introductionRoute = '/introduction';
  static const issuanceRoute = '/issuance';
  static const loginDetailRoute = '/login_detail';
  static const menuRoute = '/menu';
  static const needHelpRoute = '/menu/need_help';
  static const organizationDetailRoute = '/organization';
  static const pinBlockedRoute = '/pin/blocked';
  static const pinRoute = '/pin';
  static const pinRecoveryRoute = '/pin/recovery';
  static const pinTimeoutRoute = '/pin/timeout';
  static const policyRoute = '/policy';
  static const privacyPolicyRoute = '/privacy_policy';
  static const qrRoute = '/qr';
  static const renewPidRoute = '/pid/renew';
  static const revocationCodeRoute = '/revocation_code';
  static const reviewRevocationCodeRoute = '/review_revocation_code';
  static const settingsRoute = '/menu/settings';
  static const setupSecurityRoute = '/security/setup';
  static const signRoute = '/sign';
  static const splashRoute = '/';
  static const themeRoute = '/theme';
  static const tourOverviewRoute = '/tour';
  static const tourVideoRoute = '/tour/video';
  static const updateInfoRoute = '/update_info';
  static const walletHistoryRoute = '/wallet/history';
  static const walletPersonalizeRoute = '/wallet/personalize';
  static const walletTransferSourceRoute = '/wallet_transfer/source';
  static const walletTransferTargetRoute = '/wallet_transfer/target';
  static const walletTransferFaqRoute = '/settings/wallet_transfer_faq';
  static const manageNotificationsRoute = '/settings/manage_notifications';

  static final Map<String, WidgetBuilder Function(RouteSettings)> _routeBuilders = {
    WalletRoutes.splashRoute: (_) => _createSplashScreenBuilder,
    WalletRoutes.qrRoute: (_) => _createQrScreenBuilder,
    WalletRoutes.introductionRoute: (_) => _createIntroductionScreenBuilder,
    WalletRoutes.introductionPrivacyRoute: (_) => _createIntroductionPrivacyScreenBuilder,
    WalletRoutes.aboutRoute: (_) => _createAboutScreenBuilder,
    WalletRoutes.pinRoute: (_) => _createPinScreenBuilder,
    WalletRoutes.setupSecurityRoute: (_) => _createSetupSecurityScreenBuilder,
    WalletRoutes.menuRoute: (_) => _createMenuScreenBuilder,
    WalletRoutes.dashboardRoute: _createDashboardScreenBuilder,
    WalletRoutes.cardDetailRoute: _createCardDetailScreenBuilder,
    WalletRoutes.cardDataRoute: _createCardDataScreenBuilder,
    WalletRoutes.cardHistoryRoute: _createCardHistoryScreenBuilder,
    WalletRoutes.demoRoute: (_) => _createDemoScreenBuilder,
    WalletRoutes.contactRoute: (_) => _createContactScreenBuilder,
    WalletRoutes.themeRoute: (_) => _createThemeScreenBuilder,
    WalletRoutes.disclosureRoute: _createDisclosureScreenBuilder,
    WalletRoutes.forgotPinRoute: _createForgotPinScreenBuilder,
    WalletRoutes.policyRoute: _createPolicyScreenBuilder,
    WalletRoutes.issuanceRoute: _createIssuanceScreenBuilder,
    WalletRoutes.signRoute: _createSignScreenBuilder,
    WalletRoutes.walletPersonalizeRoute: _createWalletPersonalizeScreenBuilder,
    WalletRoutes.walletHistoryRoute: (_) => _createHistoryOverviewScreenBuilder,
    WalletRoutes.historyDetailRoute: _createHistoryDetailScreenBuilder,
    WalletRoutes.organizationDetailRoute: _createOrganizationDetailScreenBuilder,
    WalletRoutes.changeLanguageRoute: (_) => _createChangeLanguageScreenBuilder,
    WalletRoutes.changePinRoute: (_) => _createChangePinScreenBuilder,
    WalletRoutes.pinRecoveryRoute: _createPinRecoveryScreenBuilder,
    WalletRoutes.pinTimeoutRoute: _createPinTimeoutScreenBuilder,
    WalletRoutes.pinBlockedRoute: _createPinBlockedScreenBuilder,
    WalletRoutes.loginDetailRoute: _createLoginDetailScreenBuilder,
    WalletRoutes.settingsRoute: (_) => _createSettingsScreenBuilder,
    WalletRoutes.needHelpRoute: (_) => _createNeedHelpScreenBuilder,
    WalletRoutes.biometricsSettingsRoute: (_) => _createBiometricsSettingsScreenBuilder,
    WalletRoutes.privacyPolicyRoute: (_) => _createPrivacyPolicyScreenBuilder,
    WalletRoutes.updateInfoRoute: (_) => _createUpdateInfoScreenBuilder,
    WalletRoutes.tourOverviewRoute: (_) => _createTourOverviewScreenBuilder,
    WalletRoutes.tourVideoRoute: _createTourVideoScreenBuilder,
    WalletRoutes.renewPidRoute: _createRenewPidScreenBuilder,
    WalletRoutes.revocationCodeRoute: (_) => _createRevocationCodeScreenBuilder,
    WalletRoutes.reviewRevocationCodeRoute: (_) => _createReviewRevocationCodeScreenBuilder,
    WalletRoutes.walletTransferSourceRoute: _createWalletTransferSourceRoute,
    WalletRoutes.walletTransferTargetRoute: _createWalletTransferTargetRoute,
    WalletRoutes.walletTransferFaqRoute: (_) => _createWalletTransferFaqScreenBuilder,
    WalletRoutes.manageNotificationsRoute: (_) => _createManageNotificationsScreenBuilder,
  };

  static Route<dynamic> routeFactory(RouteSettings settings) {
    final builderFactory = _routeBuilders[settings.name];
    if (builderFactory == null) {
      throw UnsupportedError('Unknown route: ${settings.name}');
    }

    final builder = builderFactory(settings);
    if (publicRoutes.contains(settings.name)) {
      return MaterialPageRoute(builder: builder, settings: settings);
    }
    return SecuredPageRoute(builder: builder, settings: settings, transition: _resolvePageTransition(settings));
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

  static List<Route<dynamic>> initialRoutes(String route) => [MaterialPageRoute(builder: _createSplashScreenBuilder)];
}

Widget _createSplashScreenBuilder(BuildContext context) => BlocProvider<SplashBloc>(
  create: (BuildContext context) => SplashBloc(context.read(), context.read())..add(const InitSplashEvent()),
  child: const SplashScreen(),
);

Widget _createQrScreenBuilder(BuildContext context) => BlocProvider<QrBloc>(
  create: (BuildContext context) => QrBloc(context.read(), context.read())..add(const QrScanCheckPermission()),
  child: const QrScreen(),
);

Widget _createIntroductionScreenBuilder(BuildContext context) => const IntroductionScreen();

Widget _createIntroductionPrivacyScreenBuilder(BuildContext context) => const IntroductionPrivacyScreen();

WidgetBuilder _createForgotPinScreenBuilder(RouteSettings settings) =>
    (context) => const ForgotPinScreen();

Widget _createAboutScreenBuilder(BuildContext context) => const AboutScreen();

Widget _createPinScreenBuilder(BuildContext context) => BlocProvider<PinBloc>(
  create: (BuildContext context) => PinBloc(context.read<UnlockWalletWithPinUseCase>()),
  child: PinScreen(onUnlock: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.dashboardRoute)),
);

Widget _createSetupSecurityScreenBuilder(BuildContext context) => BlocProvider<SetupSecurityBloc>(
  create: (BuildContext context) => SetupSecurityBloc(
    context.read(),
    context.read(),
    context.read(),
    context.read(),
  ),
  child: const SetupSecurityScreen(),
);

WidgetBuilder _createDashboardScreenBuilder(RouteSettings settings) {
  final DashboardScreenArgument? argument = DashboardScreen.getArgument(settings);
  return (context) => BlocProvider(
    create: (context) => DashboardBloc(
      context.read(),
      context.read(),
      argument?.cards,
    )..add(const DashboardLoadTriggered()),
    child: const DashboardScreen(),
  );
}

Widget _createMenuScreenBuilder(BuildContext context) {
  return BlocProvider<MenuBloc>(
    create: (context) => MenuBloc(context.read(), context.read()),
    child: const MenuScreen(),
  );
}

WidgetBuilder _createCardDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    final CardDetailScreenArgument argument = CardDetailScreen.getArgument(settings);
    return BlocProvider<CardDetailBloc>(
      create: (context) =>
          CardDetailBloc(context.read(), context.read(), argument.card)..add(CardDetailLoadTriggered(argument.cardId)),
      child: CardDetailScreen(cardTitle: argument.cardTitle?.l10nValue(context)),
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
    final String attestationId = CardHistoryScreen.getArguments(settings);
    return BlocProvider<CardHistoryBloc>(
      create: (context) =>
          CardHistoryBloc(context.read(), context.read())..add(CardHistoryLoadTriggered(attestationId)),
      child: const CardHistoryScreen(),
    );
  };
}

Widget _createThemeScreenBuilder(BuildContext context) => const ThemeScreen();

WidgetBuilder _createDisclosureScreenBuilder(RouteSettings settings) {
  final args = DisclosureScreen.getArgument(settings);
  return (context) {
    return BlocProvider<DisclosureBloc>(
      create: (BuildContext context) =>
          DisclosureBloc(
            context.read(),
            context.read(),
            context.read(),
          )..add(
            DisclosureSessionStarted(
              args.uri,
              isQrCode: args.isQrCode,
            ),
          ),
      child: const DisclosureScreen(),
    );
  };
}

WidgetBuilder _createPolicyScreenBuilder(RouteSettings settings) {
  return (context) {
    final PolicyScreenArguments args = PolicyScreen.getArguments(settings);
    return PolicyScreen(
      relyingParty: args.relyingParty,
      policy: args.policy,
      showSignatureRow: args.showSignatureRow,
    );
  };
}

WidgetBuilder _createIssuanceScreenBuilder(RouteSettings settings) {
  return (context) {
    final IssuanceScreenArgument argument = IssuanceScreen.getArgument(settings);
    return BlocProvider<IssuanceBloc>(
      create: (BuildContext context) {
        return IssuanceBloc(
          context.read(),
          context.read(),
        )..add(IssuanceSessionStarted(argument.uri!, isQrCode: argument.isQrCode));
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
  final argument = Consumable(tryCast<String>(settings.arguments));
  return (context) {
    return BlocProvider<WalletPersonalizeBloc>(
      create: (BuildContext context) {
        final bloc = WalletPersonalizeBloc(
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          continueFromDigiD: argument.peek() != null,
        );
        if (argument.peek() != null) bloc.add(WalletPersonalizeContinuePidIssuance(argument.value!));
        return bloc;
      },
      child: const WalletPersonalizeScreen(),
    );
  };
}

Widget _createHistoryOverviewScreenBuilder(BuildContext context) {
  return BlocProvider<HistoryOverviewBloc>(
    create: (BuildContext context) => HistoryOverviewBloc(context.read())..add(const HistoryOverviewLoadTriggered()),
    child: const HistoryOverviewScreen(),
  );
}

WidgetBuilder _createHistoryDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    final HistoryDetailScreenArgument argument = HistoryDetailScreen.getArgument(settings);
    return BlocProvider<HistoryDetailBloc>(
      create: (BuildContext context) => HistoryDetailBloc(context.read())
        ..add(
          HistoryDetailLoadTriggered(event: argument.walletEvent),
        ),
      child: const HistoryDetailScreen(),
    );
  };
}

Widget _createChangeLanguageScreenBuilder(BuildContext context) {
  return BlocProvider<ChangeLanguageBloc>(
    create: (BuildContext context) =>
        ChangeLanguageBloc(context.read(), () => Localizations.localeOf(context))..add(ChangeLanguageLoadTriggered()),
    child: const ChangeLanguageScreen(),
  );
}

Widget _createChangePinScreenBuilder(BuildContext context) {
  return BlocProvider<ChangePinBloc>(
    create: (BuildContext context) => ChangePinBloc(context.read(), context.read()),
    child: const ChangePinScreen(),
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

WidgetBuilder _createLoginDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    final argument = LoginDetailScreen.getArgument(settings);
    return LoginDetailScreen(
      organization: argument.organization,
      policy: argument.policy,
      cardRequests: argument.cardRequests,
      sharedDataWithOrganizationBefore: argument.sharedDataWithOrganizationBefore,
    );
  };
}

WidgetBuilder _createOrganizationDetailScreenBuilder(RouteSettings settings) {
  return (context) {
    final OrganizationDetailScreenArgument argument = OrganizationDetailScreen.getArgument(settings);
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

Widget _createNeedHelpScreenBuilder(BuildContext context) => const NeedHelpScreen();

Widget _createContactScreenBuilder(BuildContext context) => const ContactScreen();

Widget _createDemoScreenBuilder(BuildContext context) => const DemoScreen();

Widget _createBiometricsSettingsScreenBuilder(BuildContext context) => BlocProvider<BiometricSettingsBloc>(
  create: (BuildContext context) {
    return BiometricSettingsBloc(
      context.read(),
      context.read(),
      context.read(),
      context.read(),
      context.read(),
    )..add(const BiometricLoadTriggered());
  },
  child: const BiometricSettingScreen(),
);

Widget _createPrivacyPolicyScreenBuilder(BuildContext context) => const PrivacyPolicyScreen();

Widget _createUpdateInfoScreenBuilder(BuildContext context) => const UpdateInfoScreen();

Widget _createTourOverviewScreenBuilder(BuildContext context) {
  return BlocProvider<TourOverviewBloc>(
    create: (BuildContext context) {
      return TourOverviewBloc(context.read(), context.read())..add(const FetchVideosEvent());
    },
    child: const TourOverviewScreen(),
  );
}

WidgetBuilder _createTourVideoScreenBuilder(RouteSettings settings) {
  return (context) {
    final TourVideoScreenArgument argument = TourVideoScreen.getArgument(settings);
    return TourVideoScreen(
      videoTitle: argument.videoTitle,
      videoUrl: argument.videoUrl,
      subtitleUrl: argument.subtitleUrl,
    );
  };
}

WidgetBuilder _createRenewPidScreenBuilder(RouteSettings settings) {
  final argument = Consumable(tryCast<String>(settings.arguments));
  return (context) {
    return BlocProvider<RenewPidBloc>(
      create: (BuildContext context) {
        final bloc = RenewPidBloc(
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          continueFromDigiD: argument.peek() != null,
        );
        if (argument.peek() != null) bloc.add(RenewPidContinuePidRenewal(argument.value!));
        return bloc;
      },
      child: const RenewPidScreen(),
    );
  };
}

WidgetBuilder _createPinRecoveryScreenBuilder(RouteSettings settings) {
  final argument = Consumable(tryCast<String>(settings.arguments));
  return (context) {
    return BlocProvider<RecoverPinBloc>(
      create: (BuildContext context) {
        final bloc = RecoverPinBloc(
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          continueFromDigiD: argument.peek() != null,
        );
        if (argument.peek() != null) bloc.add(RecoverPinContinuePinRecovery(argument.value!));
        return bloc;
      },
      child: const RecoverPinScreen(),
    );
  };
}

WidgetBuilder _createWalletTransferSourceRoute(RouteSettings settings) {
  final argument = Consumable(tryCast<String>(settings.arguments));
  return (context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<WalletTransferSourceBloc>(
          create: (BuildContext context) {
            final bloc = WalletTransferSourceBloc(
              context.read(),
              context.read(),
              context.read(),
              context.read(),
              context.read(),
            );
            if (argument.peek() != null) {
              bloc.add(WalletTransferAcknowledgeTransferEvent(argument.value!));
            } else {
              final state = WalletTransferGenericError(GenericError('No valid uri', sourceError: argument));
              bloc.add(WalletTransferUpdateStateEvent(state));
            }
            return bloc;
          },
        ),
        BlocProvider<PinBloc>(create: (BuildContext context) => PinBloc(context.read<ConfirmWalletTransferUseCase>())),
      ],
      child: const WalletTransferSourceScreen(),
    );
  };
}

WidgetBuilder _createWalletTransferTargetRoute(RouteSettings settings) {
  return (context) {
    return BlocProvider<WalletTransferTargetBloc>(
      create: (BuildContext context) {
        return WalletTransferTargetBloc(
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          context.read(),
          context.read(),
        );
      },
      child: const WalletTransferTargetScreen(),
    );
  };
}

Widget _createWalletTransferFaqScreenBuilder(BuildContext context) => const WalletTransferFaqScreen();

Widget _createRevocationCodeScreenBuilder(BuildContext context) => BlocProvider<RevocationCodeBloc>(
  create: (BuildContext context) => RevocationCodeBloc(
    context.read(),
    context.read(),
  )..add(const RevocationCodeLoadTriggered()),
  child: const RevocationCodeScreen(),
);

Widget _createReviewRevocationCodeScreenBuilder(BuildContext context) => BlocProvider<ReviewRevocationCodeBloc>(
  create: (BuildContext context) => ReviewRevocationCodeBloc(),
  child: const ReviewRevocationCodeScreen(),
);

Widget _createManageNotificationsScreenBuilder(BuildContext context) => BlocProvider<ManageNotificationsBloc>(
  create: (BuildContext context) => ManageNotificationsBloc(
    context.read(),
    context.read(),
    context.read(),
    context.read(),
    context.read(),
  )..add(const ManageNotificationsLoadTriggered()),
  child: const ManageNotificationsScreen(),
);
