import 'package:rxdart/rxdart.dart';

import '../notification_settings_store.dart';
import '../shared_preferences_provider.dart';

const _kShowNotificationRequestKey = 'show_notification_request';
const _kPushNotificationsEnabledKey = 'push_notifications_enabled';

class NotificationSettingsStoreImpl extends NotificationSettingsStore {
  final PreferenceProvider _preferences;
  final BehaviorSubject<bool> _showNotificationsEnabledSubject = BehaviorSubject();

  NotificationSettingsStoreImpl(this._preferences) {
    getPushNotificationsEnabled().then(_showNotificationsEnabledSubject.add);
  }

  @override
  Future<bool?> getShowNotificationRequestFlag() async {
    final prefs = await _preferences.call();
    return prefs.getBool(_kShowNotificationRequestKey);
  }

  @override
  Future<void> setShowNotificationRequestFlag({bool? showNotificationRequest}) async {
    final prefs = await _preferences.call();
    if (showNotificationRequest == null) {
      await prefs.remove(_kShowNotificationRequestKey);
    } else {
      await prefs.setBool(_kShowNotificationRequestKey, showNotificationRequest);
    }
  }

  @override
  Future<bool> getPushNotificationsEnabled() async {
    final preferences = await _preferences.call();
    return preferences.getBool(_kPushNotificationsEnabledKey) ?? true;
  }

  @override
  Future<void> setPushNotificationsEnabled({required bool enabled}) async {
    final preferences = await _preferences.call();
    await preferences.setBool(_kPushNotificationsEnabledKey, enabled);
    _showNotificationsEnabledSubject.add(enabled);
  }

  @override
  Stream<bool> observePushNotificationsEnabled() => _showNotificationsEnabledSubject.distinct();
}
