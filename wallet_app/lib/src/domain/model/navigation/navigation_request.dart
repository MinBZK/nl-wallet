class NavigationRequest {
  /// The destination route to navigate to
  final String destination;

  /// An optional argument to pass to the destination route
  final Object? argument;

  /// A list of navigation prerequisites, used to specify which conditions have to be met before the user should be navigated to [destination]
  final List<NavigationPrerequisite> navigatePrerequisites;

  /// A list of navigation pre navigation actions, used to specify which actions should be performed before the user should be navigated to [destination]
  final List<PreNavigationAction> preNavigationActions;

  NavigationRequest(
    this.destination, {
    this.argument,
    this.navigatePrerequisites = const [],
    this.preNavigationActions = const [],
  });

  @override
  String toString() {
    return 'NavigationRequest{destination: $destination, argument: $argument}';
  }
}

// The requirements that need to be fulfilled before the wallet can navigate
enum NavigationPrerequisite { walletUnlocked, walletInitialized, pidInitialized }

// The action that needs to be performed before navigating
enum PreNavigationAction { setupMockedWallet }
