import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../../domain/model/notification/app_notification.dart';
import '../../../../util/extension/pid_attestation_extension.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/notification_settings_store.dart';
import '../notification_repository.dart';

class NotificationRepositoryImpl implements NotificationRepository {
  final TypedWalletCore _core;
  final Mapper<core.AppNotification, AppNotification> _notificationMapper;
  final Mapper<core.NotificationType, NotificationType> _notificationTypeMapper;
  final NotificationSettingsStore _notificationSettingsStore;
  final Mapper<core.FlutterConfiguration, FlutterAppConfiguration> _flutterAppConfigurationMapper;

  NotificationRepositoryImpl(
    this._core,
    this._notificationMapper,
    this._notificationTypeMapper,
    this._notificationSettingsStore,
    this._flutterAppConfigurationMapper,
  );

  /// Observes the stream of notifications, filtered for PID duplicates.
  @override
  Stream<List<AppNotification>> observeNotifications({bool filterDuplicatePidNotifications = true}) {
    final stream = _core.observeNotifications().map(_notificationMapper.mapList);
    if (!filterDuplicatePidNotifications) return stream;
    return stream.asyncMap(_filterDuplicatePidNotifications);
  }

  /// Returns whether the notification request prompt should be shown.
  @override
  Future<bool?> getShowNotificationRequestFlag() => _notificationSettingsStore.getShowNotificationRequestFlag();

  /// Sets the flag for showing the notification request prompt.
  @override
  Future<void> setShowNotificationRequestFlag({bool? showNotificationRequest}) =>
      _notificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: showNotificationRequest);

  /// Enables or disables push notifications.
  @override
  Future<void> setPushNotificationsEnabled({required bool enabled}) =>
      _notificationSettingsStore.setPushNotificationsEnabled(enabled: enabled);

  /// Observes the push notification enabled state.
  @override
  Stream<bool> observePushNotificationsEnabled() => _notificationSettingsStore.observePushNotificationsEnabled();

  /// Returns whether push notifications are currently enabled.
  @override
  Future<bool> arePushNotificationsEnabled() async => _notificationSettingsStore.getPushNotificationsEnabled();

  /// Registers a callback for direct notifications, with PID deduplication applied.
  @override
  void setDirectNotificationCallback(
    Function(int, NotificationType) callback, {
    bool filterDuplicatePidNotifications = true,
  }) {
    _core.setupNotificationCallback((items) async {
      AppNotification toAppNotification(item) => AppNotification(
        id: item.$1,
        type: _notificationTypeMapper.map(item.$2),
        displayTargets: const [] /*ignored in this flow*/,
      );
      final mappedItems = items.map(toAppNotification).toList();

      final result = filterDuplicatePidNotifications
          ? await _filterDuplicatePidNotifications(mappedItems)
          : mappedItems;

      notify(AppNotification notification) => callback(notification.id, notification.type);
      result.forEach(notify);
    });
  }

  /// Filters out duplicate notifications for PID cards, keeping only the notification for the
  /// highest priority PID attestation as defined in the app configuration.
  Future<List<AppNotification>> _filterDuplicatePidNotifications(List<AppNotification> notifications) async {
    final config = await _core.observeConfig().map(_flutterAppConfigurationMapper.map).first;
    final pidAttestationTypes = config.pidAttestationTypes;
    final pidNotifications = notifications.where((it) => pidAttestationTypes.contains(it.type.card.attestationType));

    final result = <AppNotification>[];
    final handled = <AppNotification>{};

    for (final notification in notifications) {
      // Skip if already handled.
      if (handled.contains(notification)) continue;

      // Add if not related to PID.
      final card = notification.type.card;
      if (!pidAttestationTypes.contains(card.attestationType)) {
        result.add(notification);
        continue;
      }

      // Resolve related PID notifications.
      final relatedNotifications = pidNotifications.where((it) => it.matches(notification)).toList();
      // Find highest priority notification based on order in config.
      final priorityNotification = _findPriorityNotification(relatedNotifications, config);

      // Add the priority notification if it exists.
      if (priorityNotification != null) result.add(priorityNotification);

      // Avoid processing duplicate notifications in future loop.
      handled.addAll(relatedNotifications);
    }
    return result;
  }

  AppNotification? _findPriorityNotification(List<AppNotification> notifications, FlutterAppConfiguration config) {
    for (final pidAttestation in config.pidAttestations) {
      final match = notifications.firstWhereOrNull((n) => pidAttestation.matches(n.type.card));
      if (match != null) return match;
    }
    return notifications.firstOrNull;
  }
}

extension _AppNotificationExtensions on AppNotification {
  /// Checks if two notifications refer to the same event, ignoring card or ID.
  bool matches(AppNotification other) {
    if (this == other) return true;
    if (!const DeepCollectionEquality.unordered().equals(displayTargets, other.displayTargets)) return false;
    return type.matches(other.type);
  }
}

extension _NotificationTypeExtensions on NotificationType {
  /// Checks if two notification types refer to the same event, ignoring card details.
  bool matches(NotificationType other) {
    if (runtimeType != other.runtimeType) return false;
    final self = this;
    return switch ((self, other)) {
      (final CardExpiresSoon s, final CardExpiresSoon o) => s.expiresAt == o.expiresAt,
      (CardExpired _, CardExpired _) => true,
      (CardRevoked _, CardRevoked _) => true,
      _ => false,
    };
  }
}
