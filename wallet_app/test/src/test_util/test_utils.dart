import 'dart:ui';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';

abstract class TestUtils {
  static Future<AppLocalizations> get englishLocalizations => AppLocalizations.delegate.load(const Locale('en'));

  static Future<AppLocalizations> get dutchLocalizations => AppLocalizations.delegate.load(const Locale('nl'));

  static Future<AppLocalizations> getLocalizations(Locale locale) => AppLocalizations.delegate.load(locale);

  static void mockSensorsPlugin() {
    // Mock the accelerometer, as this is used by e.g. [CardHolograph] to make it appear reflective.
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('dev.fluttercommunity.plus/sensors/accelerometer'), (MethodCall methodCall) async {
      if (methodCall.method == 'listen') {
        return <String, dynamic>{};
      }
      return null;
    });

    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('dev.fluttercommunity.plus/sensors/method'), (MethodCall methodCall) async {
      if (methodCall.method == 'setGyroscopeSamplingPeriod') {
        return <String, dynamic>{};
      }
      return null;
    });

    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('dev.fluttercommunity.plus/sensors/gyroscope'), (MethodCall methodCall) async {
      if (methodCall.method == 'cancel') {
        return <String, dynamic>{};
      }
      return null;
    });
  }
}
