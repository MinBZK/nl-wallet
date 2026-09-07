import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/event/listener/notification_app_event_listener.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockCheckPermissionUseCase checkPermissionUseCase;
  late MockNavigationService navigationService;
  late MockNotificationRepository notificationRepository;
  late NotificationAppEventListener listener;

  setUp(() {
    checkPermissionUseCase = MockCheckPermissionUseCase();
    navigationService = MockNavigationService();
    notificationRepository = MockNotificationRepository();
    listener = NotificationAppEventListener(
      checkPermissionUseCase,
      navigationService,
      notificationRepository,
    );
  });

  /// Invokes [NotificationAppEventListener.onDashboardShown] and elapses the delay
  /// that is awaited before the permission sheet is shown.
  void showDashboard() {
    fakeAsync((async) {
      listener.onDashboardShown();
      async.elapse(const Duration(seconds: 2));
    });
  }

  group('onDashboardShown', () {
    test('should only set the flag on the first dashboard visit', () async {
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => null);

      await listener.onDashboardShown();

      verify(notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: true)).called(1);
      verifyZeroInteractions(checkPermissionUseCase);
      verifyNever(navigationService.showDialog(any));
    });

    test('should show the permission sheet on the second dashboard visit', () async {
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => true);
      when(checkPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false),
      );

      showDashboard();

      verify(notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false)).called(1);
      verify(navigationService.showDialog(.requestNotificationPermission)).called(1);
    });

    test('should not show the permission sheet again once it has been shown', () async {
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => false);

      await listener.onDashboardShown();

      verifyZeroInteractions(checkPermissionUseCase);
      verifyNever(navigationService.showDialog(any));
    });

    test('should not show the permission sheet when the permission is permanently denied', () async {
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => true);
      when(checkPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true),
      );

      await listener.onDashboardShown();

      // The flag is kept, so the sheet can still be shown if the permission is ever granted externally.
      verifyNever(notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false));
      verifyNever(navigationService.showDialog(any));
    });

    test('should show the permission sheet when permission is granted but the setting is disabled', () async {
      // This is the default state on Android < 13, where the notification permission is granted without asking.
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => true);
      when(notificationRepository.arePushNotificationsEnabled()).thenAnswer((_) async => false);
      when(checkPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false),
      );

      showDashboard();

      verify(notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false)).called(1);
      verify(navigationService.showDialog(.requestNotificationPermission)).called(1);
    });

    test('should only clear the flag when permission is granted and the setting is enabled', () async {
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => true);
      when(notificationRepository.arePushNotificationsEnabled()).thenAnswer((_) async => true);
      when(checkPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false),
      );

      await listener.onDashboardShown();

      // Notifications are already fully enabled, so the sheet is never needed again.
      verify(notificationRepository.setShowNotificationRequestFlag(showNotificationRequest: false)).called(1);
      verifyNever(navigationService.showDialog(any));
    });

    test('should show the permission sheet only once for concurrent dashboard events', () async {
      when(notificationRepository.getShowNotificationRequestFlag()).thenAnswer((_) async => true);
      when(checkPermissionUseCase.invoke(any)).thenAnswer(
        (_) async => const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false),
      );

      fakeAsync((async) {
        listener.onDashboardShown();
        listener.onDashboardShown();
        async.elapse(const Duration(seconds: 2));
      });

      verify(navigationService.showDialog(.requestNotificationPermission)).called(1);
    });
  });
}
