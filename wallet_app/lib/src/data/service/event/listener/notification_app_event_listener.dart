import '../../../../domain/app_event/app_event_listener.dart';
import '../../../../domain/usecase/permission/check_permission_usecase.dart';
import '../../../repository/notification/notification_repository.dart';
import '../../navigation_service.dart';

/// [AppEventListener] that observes events that require notification related actions
class NotificationAppEventListener extends AppEventListener {
  final CheckPermissionUseCase _checkPermissionUseCase;

  final NavigationService _navigationService;
  final NotificationRepository _notificationRepository;

  bool _requestNotificationPermission = false;

  NotificationAppEventListener(
    this._checkPermissionUseCase,
    this._navigationService,
    this._notificationRepository,
  ) {
    _checkAndSetRequestNotificationPermission();
  }

  @override
  void onWalletLocked() => _checkAndSetRequestNotificationPermission();

  Future<void> _checkAndSetRequestNotificationPermission() async {
    if (await _notificationRepository.getShowNotificationRequestFlag() ?? false) {
      final permission = await _checkPermissionUseCase.invoke(.notification);
      _requestNotificationPermission = !permission.isGranted && !permission.isPermanentlyDenied;
    }
  }

  @override
  Future<void> onDashboardShown() async {
    // Check if settings flag is set, it is null the first time we reach the dashboard.
    if (await _notificationRepository.getShowNotificationRequestFlag() == null) {
      // Set to true, so the next time we will request notification permission if needed
      await _notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: true);
    }

    // Check if flag to check for notifications is set
    if (_requestNotificationPermission) {
      // Make sure we only execute this block once
      _requestNotificationPermission = false;
      await _notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false);
      // Request permissions as per: PVW-5249
      await Future.delayed(const Duration(seconds: 2));
      await _navigationService.showDialog(.requestNotificationPermission);
    }
  }
}
