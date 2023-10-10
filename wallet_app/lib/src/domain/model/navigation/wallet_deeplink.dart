import '../../usecase/navigation/perform_pre_navigation_actions_usecase.dart';

/// Simple wrapper for a (pre)processed deeplink so we can distinguish if 'we' (aka the wallet_app) should handle
/// it or if it's not recognized and thus should be delegated to the wallet_core.
sealed class WalletDeeplink {
  final Uri uri;

  const WalletDeeplink(this.uri);
}

/// A deeplink we can parse and handle directly within `wallet_app`.
class NavigationRequestDeeplink extends WalletDeeplink {
  final NavigationRequest request;

  NavigationRequestDeeplink(this.request, super.uri);
}

/// A deeplink uri we don't directly support, to be delegated to the `wallet_core` to see if it can be handled there.
class UnknownDeeplink extends WalletDeeplink {
  UnknownDeeplink(super.uri);
}
