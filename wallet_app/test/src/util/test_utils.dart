import 'dart:ui';

import 'package:flutter/services.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';

abstract class TestUtils {
  static Future<AppLocalizations> get englishLocalizations => AppLocalizations.delegate.load(const Locale('en'));

  static Future<AppLocalizations> get dutchLocalizations => AppLocalizations.delegate.load(const Locale('nl'));

  static void mockAccelerometerPlugin() {
    // Mock the accelerometer, as this is used by e.g. [CardHolograph] to make it appear reflective.
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('dev.fluttercommunity.plus/sensors/accelerometer'), (MethodCall methodCall) async {
      if (methodCall.method == 'listen') {
        return <String, dynamic>{};
      }
      return null;
    });
  }
}
