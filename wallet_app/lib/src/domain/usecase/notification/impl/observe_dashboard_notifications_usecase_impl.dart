import '../../../../data/repository/notification/notification_repository.dart';
import '../../../../feature/banner/wallet_banner.dart';
import '../../../model/notification/app_notification.dart';
import '../observe_dashboard_notifications_usecase.dart';

class ObserveDashboardNotificationsUseCaseImpl extends ObserveDashboardNotificationsUseCase {
  final NotificationRepository _notificationRepository;

  ObserveDashboardNotificationsUseCaseImpl(this._notificationRepository);

  @override
  Stream<List<WalletBanner>> invoke() {
    return _notificationRepository.observeNotifications().map(
      (input) {
        // Filter out notifications which target the [Dashboard].
        final dashboardNotifications = input
            .where((it) => it.displayTargets.any((target) => target is Dashboard))
            .toList();
        // Map them to simple [WalletBanner], ready to be displayed
        return dashboardNotifications.map(
          (it) {
            final type = it.type;
            return switch (type) {
              CardExpiresSoon() => CardExpiresSoonBanner(card: type.card, expiresAt: type.expiresAt),
              CardExpired() => CardExpiredBanner(card: type.card),
            };
          },
        ).toList();
      },
    );
  }
}
