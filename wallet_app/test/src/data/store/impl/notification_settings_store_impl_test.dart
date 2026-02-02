import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wallet/src/data/store/impl/notification_settings_store_impl.dart';
import 'package:wallet/src/data/store/notification_settings_store.dart';

void main() {
  late NotificationSettingsStore notificationSettingsStore;
  late SharedPreferences mockSharedPreferences;

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    mockSharedPreferences = await SharedPreferences.getInstance();

    notificationSettingsStore = NotificationSettingsStoreImpl(() async => mockSharedPreferences);
  });

  tearDown(() async {
    await mockSharedPreferences.clear();
    SharedPreferences.resetStatic();
  });

  test('getShowNotificationRequestFlag should return null initially', () async {
    final result = await notificationSettingsStore.getShowNotificationRequestFlag();
    expect(result, isNull);
  });

  test('setShowNotificationRequestFlag and getShowNotificationRequestFlag should work correctly', () async {
    await notificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: true);
    var result = await notificationSettingsStore.getShowNotificationRequestFlag();
    expect(result, true);

    await notificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: false);
    result = await notificationSettingsStore.getShowNotificationRequestFlag();
    expect(result, false);
  });

  test('setShowNotificationRequestFlag with null should remove the flag', () async {
    await notificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: true);
    await notificationSettingsStore.setShowNotificationRequestFlag(showNotificationRequest: null);
    final result = await notificationSettingsStore.getShowNotificationRequestFlag();
    expect(result, isNull);
  });

  test('getPushNotificationsEnabled should return true by default', () async {
    final result = await notificationSettingsStore.getPushNotificationsEnabled();
    expect(result, true);
  });

  test('setPushNotificationsEnabled and getPushNotificationsEnabled should work correctly', () async {
    await notificationSettingsStore.setPushNotificationsEnabled(enabled: false);
    var result = await notificationSettingsStore.getPushNotificationsEnabled();
    expect(result, false);

    await notificationSettingsStore.setPushNotificationsEnabled(enabled: true);
    result = await notificationSettingsStore.getPushNotificationsEnabled();
    expect(result, true);
  });

  test('observePushNotificationsEnabled should emit changes', () async {
    final completer = Completer<void>();
    final stream = notificationSettingsStore.observePushNotificationsEnabled();

    // Expect initial value
    expect(stream, emits(true));

    // Set new value and expect it to be emitted
    final subscription = stream.skip(1).listen((event) {
      expect(event, false);
      completer.complete();
    });

    await notificationSettingsStore.setPushNotificationsEnabled(enabled: false);
    await completer.future;
    await subscription.cancel();
  });

  test('observePushNotificationsEnabled should not emit duplicates', () async {
    final completer = Completer<void>();
    final stream = notificationSettingsStore.observePushNotificationsEnabled();
    int emissionCount = 0;

    final subscription = stream.listen((event) {
      emissionCount++;
      if (emissionCount == 2) {
        // After the initial emission and one change
        completer.complete();
      }
    });

    await notificationSettingsStore.setPushNotificationsEnabled(enabled: true); // Set the same (initial) value again
    await notificationSettingsStore.setPushNotificationsEnabled(enabled: true); // Set the same (initial) value again
    await notificationSettingsStore.setPushNotificationsEnabled(enabled: true); // Set the same (initial) value again
    await notificationSettingsStore.setPushNotificationsEnabled(enabled: false);
    await completer.future;
    expect(emissionCount, 2); // Should only have emitted twice (initial and the change)
    await subscription.cancel();
  });
}
