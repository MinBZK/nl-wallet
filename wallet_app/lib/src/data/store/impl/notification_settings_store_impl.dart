import '../notification_settings_store.dart';
import '../shared_preferences_provider.dart';

const _kShowNotificationRequestKey = 'show_notification_request';

class NotificationSettingsStoreImpl extends NotificationSettingsStore {
  final PreferenceProvider _preferences;

  NotificationSettingsStoreImpl(this._preferences);

  @override
  Future<bool?> getShowNotificationRequest() async {
    final prefs = await _preferences.call();
    return prefs.getBool(_kShowNotificationRequestKey);
  }

  @override
  Future<void> setShowNotificationRequest({bool? showNotificationRequest}) async {
    final prefs = await _preferences.call();
    if (showNotificationRequest == null) {
      await prefs.remove(_kShowNotificationRequestKey);
    } else {
      await prefs.setBool(_kShowNotificationRequestKey, showNotificationRequest);
    }
  }
}
