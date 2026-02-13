import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:timezone/data/latest_all.dart' as tz;
import 'package:timezone/timezone.dart' as tz;

import '../../domain/model/notification/notification_channel.dart';
import '../../domain/model/notification/os_notification.dart';
import '../../domain/usecase/notification/observe_os_notifications_usecase.dart';
import '../../domain/usecase/notification/set_direct_os_notification_callback_usecase.dart';
import '../../util/builder/notification/notification_payload_parser.dart';
import '../../util/extension/locale_extension.dart';
import '../../util/extension/object_extension.dart';
import '../store/active_locale_provider.dart';
import 'navigation_service.dart';

const kAndroidInitSettings = AndroidInitializationSettings('ic_notification');
const kDarwinInitSettings = DarwinInitializationSettings(
  requestAlertPermission: false,
  requestBadgePermission: false,
  requestCriticalPermission: false,
  requestSoundPermission: false,
  requestProvisionalPermission: false,
  requestProvidesAppNotificationSettings: false,
);

/// A function that provides an instance of [FlutterLocalNotificationsPlugin]. Useful for testing.
typedef LocalNotificationsPluginProvider = FlutterLocalNotificationsPlugin Function();

/// A service responsible for scheduling and managing local device notifications.
///
/// This service initializes the `flutter_local_notifications` plugin, sets up
/// platform-specific configurations, and listens for updates from [ObserveOsNotificationsUseCase]
/// to schedule or cancel notifications accordingly.
class LocalNotificationService {
  late FlutterLocalNotificationsPlugin _plugin;
  final NavigationService _navigationService;
  final ObserveOsNotificationsUseCase _observeOsNotificationsUseCase;
  final SetDirectOsNotificationCallbackUsecase _setDirectOsNotificationCallbackUsecase;
  final ActiveLocaleProvider _activeLocaleProvider;

  StreamSubscription? _notificationStreamSubscription;

  LocalNotificationService(
    this._observeOsNotificationsUseCase,
    this._setDirectOsNotificationCallbackUsecase,
    this._activeLocaleProvider,
    this._navigationService, {
    LocalNotificationsPluginProvider? factory,
  }) {
    // Resolve the plugin, fall back to default implementation
    _plugin = factory?.call() ?? FlutterLocalNotificationsPlugin();
    // Initialize TimeZones, used when scheduling
    tz.initializeTimeZones();

    _initPlugin();
  }

  Future<void> _initPlugin() async {
    await _plugin.initialize(
      const InitializationSettings(android: kAndroidInitSettings, iOS: kDarwinInitSettings),
      onDidReceiveNotificationResponse: onDidReceiveNotificationResponse,
    );

    /// Set observer to handle 'scheduled' os notifications
    _notificationStreamSubscription = _observeOsNotificationsUseCase.invoke().listen(_onNotificationUpdate);

    /// Set callback to handle 'direct' os notifications
    _setDirectOsNotificationCallbackUsecase.invoke(_onDirectNotification);

    /// Check if app was launched through notification, and handle accordingly
    final launchDetails = await _plugin.getNotificationAppLaunchDetails();
    if (launchDetails?.didNotificationLaunchApp ?? false) {
      _processPayload(launchDetails?.notificationResponse?.payload);
    }
  }

  /// Callback for when a notification is tapped by the user while the app is in the foreground.
  void onDidReceiveNotificationResponse(NotificationResponse details) {
    Fimber.d('onDidReceiveNotificationResponse: ${details.id}:${details.payload}');
    _processPayload(details.payload);
  }

  void _processPayload(String? payload) {
    NotificationPayloadParser.parse(payload)?.let((navRequest) {
      _navigationService.handleNavigationRequest(navRequest, queueIfNotReady: true);
    });
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
            payload: notification.payload,
          )
          .onError(
            (ex, stack) {
              Fimber.d('Failed to schedule ${notification.id}', ex: ex, stacktrace: stack);
            },
          );
    }
    Fimber.d('Finished (re)scheduling ${notifications.length} notifications');
  }

  void _onDirectNotification(OsNotification notification) {
    final details = NotificationDetails(
      android: _resolveAndroidDetails(notification.channel),
      iOS: const DarwinNotificationDetails(presentAlert: true),
    );
    _plugin.show(notification.id, notification.title, notification.body, details, payload: notification.payload);
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
