import '../../../domain/model/navigation/navigation_request.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../wallet_constants.dart';

/// A utility class responsible for decoding deep-link payloads from notifications.
///
/// It reverses the logic in [NotificationPayloadBuilder], converting a URI string
/// into a type-safe [NavigationRequest] object.
class NotificationPayloadParser {
  /// Parses a raw string payload into a [NavigationRequest].
  /// Returns `null` if the payload is malformed or unsupported.
  static NavigationRequest? parse(String? payload) {
    if (payload == null) return null;

    final uri = Uri.tryParse(payload);
    if (uri == null || uri.scheme != kNotificationPayloadScheme) return null;

    // Match on path
    switch (uri.path) {
      case WalletRoutes.cardDetailRoute:
        final id = uri.queryParameters['id'];
        if (id != null) return NavigationRequest.cardDetail(id);
    }

    return null;
  }
}
