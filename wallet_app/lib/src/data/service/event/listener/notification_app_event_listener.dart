import '../../../../domain/app_event/app_event_listener.dart';
import '../../../../domain/usecase/permission/check_permission_usecase.dart';
import '../../../repository/notification/notification_repository.dart';
import '../../navigation_service.dart';

/// [AppEventListener] that observes events that require notification related actions
class NotificationAppEventListener extends AppEventListener {
  final CheckPermissionUseCase _checkPermissionUseCase;

  final NavigationService _navigationService;
  final NotificationRepository _notificationRepository;

  /// Guards against concurrent [onDashboardShown] invocations showing the sheet twice.
  bool _handlingDashboardShown = false;

  /// Whether the dashboard is currently shown, the sheet should only appear on top of it.
  bool _dashboardVisible = false;

  NotificationAppEventListener(
    this._checkPermissionUseCase,
    this._navigationService,
    this._notificationRepository,
  );

  @override
  Future<void> onDashboardShown() async {
    _dashboardVisible = true;
    if (_handlingDashboardShown) return;
    _handlingDashboardShown = true;
    try {
      await _requestNotificationPermissionIfNeeded();
    } finally {
      _handlingDashboardShown = false;
    }
  }

  @override
  void onDashboardHidden() => _dashboardVisible = false;

  Future<void> _requestNotificationPermissionIfNeeded() async {
    // The flag is null the first time we reach the dashboard. In that case only flag that the
    // permission request should be shown on the next (i.e. second) dashboard visit.
    final showRequestFlag = await _notificationRepository.getShowNotificationRequestFlag();
    if (showRequestFlag == null) {
      await _notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: true);
      return;
    }
    if (!showRequestFlag) return;

    final permission = await _checkPermissionUseCase.invoke([.notification]);
    // Requesting the permission is futile when it's permanently denied. Keep the flag so the
    // request can still be shown if the permission is ever granted through the OS settings.
    if (permission.isPermanentlyDenied) return;

    // The OS permission can be granted without the user ever being asked (e.g. Android < 13 grants
    // it by default). The sheet is then still relevant, as it also enables the in-app push
    // notifications setting, so only skip it when that setting is already enabled.
    if (permission.isGranted && await _notificationRepository.arePushNotificationsEnabled()) {
      await _notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false);
      return;
    }

    // Request permissions as per: PVW-5249
    await Future.delayed(const Duration(seconds: 2));
    // The dialog is shown on top of whatever screen is visible, so without this check it would cover
    // a flow the user started from the dashboard in the meantime. Keep the flag to retry on the next visit.
    if (!_dashboardVisible) return;

    await _notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false);
    await _navigationService.showDialog(.requestNotificationPermission);
  }
}
