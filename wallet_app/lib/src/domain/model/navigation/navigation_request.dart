class NavigationRequest {
  final String destination;
  final Object? argument;

  NavigationRequest(
    this.destination, {
    this.argument,
  });

  @override
  String toString() {
    return 'NavigationRequest{destination: $destination, argument: $argument}';
  }
}
