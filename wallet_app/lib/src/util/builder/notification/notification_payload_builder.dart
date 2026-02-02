import '../../../domain/model/notification/notification_type.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../wallet_constants.dart';

/// A utility class responsible for generating deep-link payloads for notifications.
///
/// These payloads follow the custom scheme format:
/// `nlwallet://app/<route>?<query-parameters>`
///
/// This centralized builder ensures that all app notifications point to valid
/// [WalletRoutes] and provides a consistent way to pass necessary arguments
/// (like IDs) to the navigation layer.
class NotificationPayloadBuilder {
  static const _kHost = 'app';

  static String build(NotificationType type) {
    return switch (type) {
      CardExpiresSoon(:final card) || CardExpired(:final card) || CardRevoked(:final card) => Uri(
        scheme: kNotificationPayloadScheme,
        host: _kHost,
        path: WalletRoutes.cardDetailRoute,
        queryParameters: {'id': card.attestationId},
      ).toString(),
    };
  }
}
