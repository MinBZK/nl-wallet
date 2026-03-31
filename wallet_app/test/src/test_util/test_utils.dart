import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/wallet_assets.dart';

abstract class TestUtils {
  static Future<AppLocalizations> get englishLocalizations => AppLocalizations.delegate.load(const Locale('en'));

  static Future<AppLocalizations> get dutchLocalizations => AppLocalizations.delegate.load(const Locale('nl'));

  static Future<AppLocalizations> getLocalizations(Locale locale) => AppLocalizations.delegate.load(locale);

  /// Helper method to pre-cache the wallet logo asset. Needed to make sure the QrCode
  /// is able to render the embedded wallet logo in golden tests.
  static Future<void> preCacheWalletLogoForQrImageView(WidgetTester tester) async {
    final context = tester.element(find.byType(QrImageView));
    await tester.runAsync(() async {
      await precacheImage(const AssetImage(WalletAssets.logo_wallet), context);
      await precacheImage(const AssetImage(WalletAssets.logo_wallet_qr), context);
    });
    await tester.pumpAndSettle();
  }

  static void mockSensorsPlugin() {
    // Mock the accelerometer, as this is used by e.g. [CardHolograph] to make it appear reflective.
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      const MethodChannel('dev.fluttercommunity.plus/sensors/accelerometer'),
      (MethodCall methodCall) async {
        if (methodCall.method == 'listen') {
          return <String, dynamic>{};
        }
        return null;
      },
    );

    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      const MethodChannel('dev.fluttercommunity.plus/sensors/method'),
      (MethodCall methodCall) async {
        if (methodCall.method == 'setGyroscopeSamplingPeriod') {
          return <String, dynamic>{};
        }
        return null;
      },
    );

    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      const MethodChannel('dev.fluttercommunity.plus/sensors/gyroscope'),
      (MethodCall methodCall) async {
        if (methodCall.method == 'cancel') {
          return <String, dynamic>{};
        }
        return null;
      },
    );
  }
}
