import 'package:equatable/equatable.dart';

import '../../../feature/disclosure/argument/disclosure_screen_argument.dart';
import '../../../navigation/wallet_routes.dart';
import 'navigation_prerequisite.dart';
import 'pre_navigation_action.dart';

export 'navigation_prerequisite.dart';
export 'pre_navigation_action.dart';

sealed class NavigationRequest extends Equatable {
  /// The destination route to navigate to
  final String destination;

  /// An optional argument to pass to the destination route
  final Object? argument;

  /// A list of navigation prerequisites, used to specify which conditions have to be met before the user should be navigated to [destination]
  final List<NavigationPrerequisite> navigatePrerequisites;

  /// A list of navigation pre navigation actions, used to specify which actions should be performed before the user should be navigated to [destination]
  final List<PreNavigationAction> preNavigationActions;

  const NavigationRequest(
    this.destination, {
    this.argument,
    this.navigatePrerequisites = const [],
    this.preNavigationActions = const [],
  });

  @override
  String toString() {
    return 'NavigationRequest{destination: $destination, argument: $argument}';
  }

  @override
  List<Object?> get props => [destination, argument, navigatePrerequisites, preNavigationActions];
}

class GenericNavigationRequest extends NavigationRequest {
  const GenericNavigationRequest(
    String destination, {
    Object? argument,
    List<NavigationPrerequisite> navigatePrerequisites = const [],
    List<PreNavigationAction> preNavigationActions = const [],
  }) : super(
          destination,
          argument: argument,
          navigatePrerequisites: navigatePrerequisites,
          preNavigationActions: preNavigationActions,
        );
}

class PidIssuanceNavigationRequest extends NavigationRequest {
  PidIssuanceNavigationRequest(String uri)
      : super(
          WalletRoutes.walletPersonalizeRoute,
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

class DisclosureNavigationRequest extends NavigationRequest {
  DisclosureNavigationRequest(String uri)
      : super(
          WalletRoutes.disclosureRoute,
          argument: DisclosureScreenArgument(uri: uri),
          navigatePrerequisites: [
            NavigationPrerequisite.walletUnlocked,
            NavigationPrerequisite.walletInitialized,
            NavigationPrerequisite.pidInitialized,
          ],
        );
}
