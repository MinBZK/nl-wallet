import 'package:equatable/equatable.dart';

import '../../../feature/disclosure/argument/disclosure_screen_argument.dart';
import '../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../feature/sign/argument/sign_screen_argument.dart';
import '../../../navigation/wallet_routes.dart';
import 'navigation_prerequisite.dart';
import 'pre_navigation_action.dart';

export 'navigation_prerequisite.dart';
export 'pre_navigation_action.dart';

/// Common set of prerequisites: check if the wallet is unlocked, initialized and contains the pid.
const unlockedWithPidAndReadyPrerequisites = [
  NavigationPrerequisite.walletUnlocked,
  NavigationPrerequisite.walletInitialized,
  NavigationPrerequisite.pidInitialized,
  NavigationPrerequisite.walletInReadyState,
];

sealed class NavigationRequest extends Equatable {
  /// The destination route to navigate to
  final String destination;

  /// The destination that should be below the new [destination]
  final String? removeUntil;

  /// An optional argument to pass to the destination route
  final Object? argument;

  /// A list of navigation prerequisites, used to specify which conditions have to be met before the user should be navigated to [destination]
  final List<NavigationPrerequisite> navigatePrerequisites;

  /// A list of navigation pre navigation actions, used to specify which actions should be performed before the user should be navigated to [destination]
  final List<PreNavigationAction> preNavigationActions;

  const NavigationRequest(
    this.destination, {
    this.argument,
    this.removeUntil,
    this.navigatePrerequisites = const [],
    this.preNavigationActions = const [],
  });

  @override
  String toString() => 'NavigationRequest{destination: $destination, removeUntil: $removeUntil, argument: $argument}';

  @override
  List<Object?> get props => [destination, removeUntil, argument, navigatePrerequisites, preNavigationActions];

  factory NavigationRequest.dashboard() => DashboardNavigationRequest();

  factory NavigationRequest.pidIssuance(String uri) => PidIssuanceNavigationRequest(uri);

  factory NavigationRequest.pidRenewal(String uri) => PidRenewalNavigationRequest(uri);

  factory NavigationRequest.pinRecovery(String uri) => PinRecoveryNavigationRequest(uri);

  factory NavigationRequest.disclosure(String uri, {bool isQrCode = false}) =>
      DisclosureNavigationRequest(uri, isQrCode: isQrCode);

  factory NavigationRequest.issuance(String uri, {bool isQrCode = false, bool isRefreshFlow = false}) =>
      IssuanceNavigationRequest(uri, isQrCode: isQrCode, isRefreshFlow: isRefreshFlow);

  factory NavigationRequest.sign(String uri) => SignNavigationRequest(uri);

  factory NavigationRequest.walletTransferSource(String uri) => GenericNavigationRequest(
    WalletRoutes.walletTransferSourceRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    argument: uri,
    navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
  );

  factory NavigationRequest.walletTransferTarget({bool isRetry = false}) => GenericNavigationRequest(
    WalletRoutes.walletTransferTargetRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    argument: isRetry,
    navigatePrerequisites: const [
      NavigationPrerequisite.walletUnlocked,
      NavigationPrerequisite.walletInitialized,
      NavigationPrerequisite.pidInitialized,
    ],
  );
}

class GenericNavigationRequest extends NavigationRequest {
  const GenericNavigationRequest(
    super.destination, {
    super.removeUntil,
    super.argument,
    super.navigatePrerequisites,
    super.preNavigationActions,
  });
}

class DashboardNavigationRequest extends NavigationRequest {
  DashboardNavigationRequest()
    : super(
        WalletRoutes.dashboardRoute,
        removeUntil: WalletRoutes.splashRoute,
        navigatePrerequisites: [
          NavigationPrerequisite.pidInitialized,
        ],
      );
}

class PidIssuanceNavigationRequest extends NavigationRequest {
  PidIssuanceNavigationRequest(String uri)
    : super(
        WalletRoutes.walletPersonalizeRoute,
        removeUntil: WalletRoutes.splashRoute,
        argument: uri,
        navigatePrerequisites: [
          NavigationPrerequisite.walletUnlocked,
          NavigationPrerequisite.walletInitialized,
        ],
        preNavigationActions: [
          PreNavigationAction.disableUpcomingPageTransition,
        ],
      );
}

class PidRenewalNavigationRequest extends NavigationRequest {
  PidRenewalNavigationRequest(String uri)
    : super(
        WalletRoutes.renewPidRoute,
        removeUntil: WalletRoutes.cardDetailRoute,
        argument: uri,
        navigatePrerequisites: [
          NavigationPrerequisite.walletUnlocked,
          NavigationPrerequisite.walletInitialized,
        ],
        preNavigationActions: [
          PreNavigationAction.disableUpcomingPageTransition,
        ],
      );
}

class PinRecoveryNavigationRequest extends NavigationRequest {
  PinRecoveryNavigationRequest(String uri)
    : super(
        WalletRoutes.pinRecoveryRoute,
        removeUntil: WalletRoutes.forgotPinRoute,
        argument: uri,
        navigatePrerequisites: [NavigationPrerequisite.walletInitialized],
        preNavigationActions: [PreNavigationAction.disableUpcomingPageTransition],
      );
}

class DisclosureNavigationRequest extends NavigationRequest {
  DisclosureNavigationRequest(String uri, {bool isQrCode = false})
    : super(
        WalletRoutes.disclosureRoute,
        removeUntil: WalletRoutes.dashboardRoute,
        argument: DisclosureScreenArgument(uri: uri, isQrCode: isQrCode),
        navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
      );
}

class IssuanceNavigationRequest extends NavigationRequest {
  IssuanceNavigationRequest(String uri, {bool isQrCode = false, bool isRefreshFlow = false})
    : super(
        WalletRoutes.issuanceRoute,
        removeUntil: WalletRoutes.dashboardRoute,
        argument: IssuanceScreenArgument(uri: uri, isQrCode: isQrCode, isRefreshFlow: isRefreshFlow),
        navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
      );
}

class SignNavigationRequest extends NavigationRequest {
  SignNavigationRequest(String uri)
    : super(
        WalletRoutes.signRoute,
        removeUntil: WalletRoutes.dashboardRoute,
        argument: SignScreenArgument(uri: uri),
        navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
      );
}
