class NavigationRequest {
  /// The destination route to navigate to
  final String destination;

  /// An optional argument to pass to the destination route
  final Object? argument;

  /// An optional navigation prerequisite type for a certain deep -link/dive scenario (before navigation)
  final NavigationPrerequisite? navigatePrerequisite;

  NavigationRequest(
    this.destination, {
    this.argument,
    this.navigatePrerequisite,
  });

  @override
  String toString() {
    return 'NavigationRequest{destination: $destination, argument: $argument}';
  }
}

enum NavigationPrerequisite {
  setupMockedWallet,
}
