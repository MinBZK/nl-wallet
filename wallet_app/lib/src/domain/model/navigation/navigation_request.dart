import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../feature/blocked/argument/app_blocked_screen_argument.dart';
import '../../../feature/card/detail/argument/card_detail_screen_argument.dart';
import '../../../feature/disclosure/argument/disclosure_screen_argument.dart';
import '../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../feature/sign/argument/sign_screen_argument.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../wallet_core/error/core_error.dart';
import 'navigation_prerequisite.dart';
import 'pre_navigation_action.dart';

export 'navigation_prerequisite.dart';
export 'pre_navigation_action.dart';

part 'navigation_request.freezed.dart';
part 'navigation_request.g.dart';

/// Common set of prerequisites: checks if the wallet is unlocked, initialized, has a pid and in the 'ready' state.
const unlockedWithPidAndReadyPrerequisites = [
  NavigationPrerequisite.walletUnlocked,
  NavigationPrerequisite.walletInitialized,
  NavigationPrerequisite.pidInitialized,
  NavigationPrerequisite.walletInReadyState,
];

@freezed
abstract class NavigationRequest with _$NavigationRequest {
  const NavigationRequest._();

  factory NavigationRequest.fromJson(Map<String, dynamic> json) => _$NavigationRequestFromJson(json);

  const factory NavigationRequest.generic(
    String destination, {
    String? removeUntil,
    Object? argument,
    @Default([]) List<NavigationPrerequisite> navigatePrerequisites,
    @Default([]) List<PreNavigationAction> preNavigationActions,
  }) = GenericNavigationRequest;

  factory NavigationRequest.dashboard({
    Object? argument,
  }) => NavigationRequest.generic(
    WalletRoutes.dashboardRoute,
    removeUntil: WalletRoutes.splashRoute,
    argument: argument,
    navigatePrerequisites: const [NavigationPrerequisite.pidInitialized],
  );

  factory NavigationRequest.pidIssuance(String uri) => NavigationRequest.generic(
    WalletRoutes.walletPersonalizeRoute,
    removeUntil: WalletRoutes.splashRoute,
    argument: uri,
    navigatePrerequisites: const [
      NavigationPrerequisite.walletUnlocked,
      NavigationPrerequisite.walletInitialized,
    ],
    preNavigationActions: const [PreNavigationAction.disableUpcomingPageTransition],
  );

  factory NavigationRequest.pidRenewal(String uri) => NavigationRequest.generic(
    WalletRoutes.renewPidRoute,
    removeUntil: WalletRoutes.cardDetailRoute,
    argument: uri,
    navigatePrerequisites: const [
      NavigationPrerequisite.walletUnlocked,
      NavigationPrerequisite.walletInitialized,
    ],
    preNavigationActions: const [PreNavigationAction.disableUpcomingPageTransition],
  );

  factory NavigationRequest.pinRecovery(String uri) => NavigationRequest.generic(
    WalletRoutes.pinRecoveryRoute,
    removeUntil: WalletRoutes.forgotPinRoute,
    argument: uri,
    navigatePrerequisites: const [NavigationPrerequisite.walletInitialized],
    preNavigationActions: const [PreNavigationAction.disableUpcomingPageTransition],
  );

  factory NavigationRequest.disclosure({
    required DisclosureScreenArgument argument,
  }) => NavigationRequest.generic(
    WalletRoutes.disclosureRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    argument: argument,
    navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
  );

  factory NavigationRequest.issuance({
    required IssuanceScreenArgument argument,
  }) => NavigationRequest.generic(
    WalletRoutes.issuanceRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    argument: argument,
    navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
  );

  factory NavigationRequest.sign({
    required SignScreenArgument argument,
  }) => NavigationRequest.generic(
    WalletRoutes.signRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    argument: argument,
    navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
  );

  factory NavigationRequest.cardDetail(String attestationId) => NavigationRequest.generic(
    WalletRoutes.cardDetailRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
    argument: CardDetailScreenArgument.fromId(attestationId, const {}).toJson(),
  );

  factory NavigationRequest.walletTransferSource(String uri) => NavigationRequest.generic(
    WalletRoutes.walletTransferSourceRoute,
    removeUntil: WalletRoutes.dashboardRoute,
    argument: uri,
    navigatePrerequisites: unlockedWithPidAndReadyPrerequisites,
  );

  factory NavigationRequest.walletTransferTarget() => const NavigationRequest.generic(
    WalletRoutes.walletTransferTargetRoute,
    removeUntil: WalletRoutes.splashRoute,
    navigatePrerequisites: [
      NavigationPrerequisite.walletUnlocked,
      NavigationPrerequisite.walletInitialized,
      NavigationPrerequisite.pidInitialized,
    ],
  );

  factory NavigationRequest.appBlocked({required RevocationReason reason}) => NavigationRequest.generic(
    WalletRoutes.appBlockedRoute,
    removeUntil: WalletRoutes.splashRoute,
    navigatePrerequisites: [],
    argument: AppBlockedScreenArgument(reason: reason).toJson(),
  );
}
