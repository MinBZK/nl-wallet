import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:timezone/data/latest_all.dart' as tz;
import 'package:timezone/timezone.dart' as tz;

import '../../domain/model/notification/notification_channel.dart';
import '../../domain/model/notification/os_notification.dart';
import '../../domain/usecase/notification/observe_os_notifications_usecase.dart';
import '../../util/extension/locale_extension.dart';
import '../store/active_locale_provider.dart';

const kAndroidInitSettings = AndroidInitializationSettings('ic_notification');
const kDarwinInitSettings = DarwinInitializationSettings(
  requestAlertPermission: false,
  requestBadgePermission: false,
  requestCriticalPermission: false,
  requestSoundPermission: false,
  requestProvisionalPermission: false,
  requestProvidesAppNotificationSettings: false,
);

/// A service responsible for scheduling and managing local device notifications.
///
/// This service initializes the `flutter_local_notifications` plugin, sets up
/// platform-specific configurations, and listens for updates from [ObserveOsNotificationsUseCase]
/// to schedule or cancel notifications accordingly.
class LocalNotificationService {
  final FlutterLocalNotificationsPlugin _plugin = FlutterLocalNotificationsPlugin();
  final ObserveOsNotificationsUseCase _observeOsNotificationsUseCase;
  final ActiveLocaleProvider _activeLocaleProvider;

  StreamSubscription? _notificationStreamSubscription;

  LocalNotificationService(this._observeOsNotificationsUseCase, this._activeLocaleProvider) {
    // Initialize TimeZones, used when scheduling
    tz.initializeTimeZones();
    // Configure plugin initialization settings, mostly to avoid instantly requesting notifications
    final InitializationSettings initializationSettings = const InitializationSettings(
      android: kAndroidInitSettings,
      iOS: kDarwinInitSettings,
    );

    _plugin
        .initialize(
          initializationSettings,
          onDidReceiveNotificationResponse: onDidReceiveNotificationResponse,
          onDidReceiveBackgroundNotificationResponse: onDidReceiveBackgroundNotificationResponse,
        )
        .then((_) {
          _notificationStreamSubscription = _observeOsNotificationsUseCase.invoke().listen(_onNotificationUpdate);
        });
  }

  /// Callback for when a notification is tapped by the user while the app is in the foreground.
  void onDidReceiveNotificationResponse(NotificationResponse details) {
    Fimber.d('onDidReceiveNotificationResponse: ${details.id}:${details.payload}');
  }

  /// Callback for when a notification is tapped by the user while the app is in the background.
  ///
  /// This method runs in a separate isolate.
  @pragma('vm:entry-point')
  static void onDidReceiveBackgroundNotificationResponse(NotificationResponse details) {
    if (kDebugMode) print('onDidReceiveBackgroundNotificationResponse: ${details.id}:${details.payload}');
  }

  /// Handles updates to the list of operating system notifications.
  ///
  /// This method cancels all previously scheduled notifications and then
  /// schedules a new batch based on the provided list (naive approach).
  Future<void> _onNotificationUpdate(List<OsNotification> notifications) async {
    await _plugin.cancelAllPendingNotifications();
    for (final notification in notifications) {
      await _plugin
          .zonedSchedule(
            notification.id,
            notification.title,
            notification.body,
            tz.TZDateTime.from(
              notification.notifyAt,
              tz.getLocation('Europe/Amsterdam'),
            ),
            NotificationDetails(
              android: _resolveAndroidDetails(notification.channel),
              iOS: const DarwinNotificationDetails(presentAlert: true),
            ),
            androidScheduleMode: AndroidScheduleMode.inexact,
          )
          .onError(
            (ex, stack) {
              Fimber.d('Failed to schedule ${notification.id}', ex: ex, stacktrace: stack);
            },
          );
    }
    Fimber.d('Finished (re)scheduling ${notifications.length} notifications');
  }

  AndroidNotificationDetails? _resolveAndroidDetails(NotificationChannel channel) {
    return switch (channel) {
      NotificationChannel.cardUpdates => AndroidNotificationDetails(
        channel.name,
        _activeLocaleProvider.activeLocale.l10n.cardNotificationsChannelName,
        channelDescription: _activeLocaleProvider.activeLocale.l10n.cardNotificationsChannelDescription,
        autoCancel: true,
      ),
    };
  }

  /// Disposes the service and cancels the stream subscription.
  void dispose() => _notificationStreamSubscription?.cancel();
}
