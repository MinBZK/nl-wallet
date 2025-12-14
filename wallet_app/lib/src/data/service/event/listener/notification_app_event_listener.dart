import '../../../../domain/app_event/app_event_listener.dart';
import '../../../../domain/usecase/permission/check_permission_usecase.dart';
import '../../../store/notification_settings_store.dart';
import '../../navigation_service.dart';

/// [AppEventListener] that observes events that require notification related actions
class NotificationAppEventListener extends AppEventListener {
  final CheckPermissionUseCase _checkPermissionUseCase;

  final NavigationService _navigationService;
  final NotificationSettingsStore _notificationSettingsStore;

  bool _requestNotificationPermission = false;

  NotificationAppEventListener(
    this._checkPermissionUseCase,
    this._navigationService,
    this._notificationSettingsStore,
  ) {
    _checkAndSetRequestNotificationPermission();
  }

  @override
  void onWalletLocked() => _checkAndSetRequestNotificationPermission();

  Future<void> _checkAndSetRequestNotificationPermission() async {
    if (await _notificationSettingsStore.getShowNotificationRequest() ?? false) {
      final permission = await _checkPermissionUseCase.invoke(.notification);
      _requestNotificationPermission = !permission.isGranted && !permission.isPermanentlyDenied;
    }
  }

  @override
  Future<void> onDashboardShown() async {
    // Check if settings flag is set, it is null the first time we reach the dashboard.
    if (await _notificationSettingsStore.getShowNotificationRequest() == null) {
      // Set to true, so the next time we will request notification permission if needed
      await _notificationSettingsStore.setShowNotificationRequest(showNotificationRequest: true);
    }

    // Check if flag to check for notifications is set
    if (_requestNotificationPermission) {
      // Make sure we only execute this block once
      _requestNotificationPermission = false;
      await _notificationSettingsStore.setShowNotificationRequest(showNotificationRequest: false);
      // Request permissions as per: PVW-5249
      await Future.delayed(const Duration(seconds: 2));
      await _navigationService.showDialog(.requestNotificationPermission);
    }
  }
}
