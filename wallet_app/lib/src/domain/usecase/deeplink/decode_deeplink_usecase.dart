import '../../model/navigation/navigation_request.dart';

abstract class DecodeDeeplinkUseCase {
  /// The host name for QR deeplink
  String get deeplinkHost;

  /// The host name for (E2E) testing deeplink
  String get deepDiveHost;

  NavigationRequest? invoke(Uri uri);
}
