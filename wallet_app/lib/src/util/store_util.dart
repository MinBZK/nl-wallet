import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:package_info_plus/package_info_plus.dart';

import '../../environment.dart';
import 'launch_util.dart';

const _testFlightUrl = 'itms-beta://';
const _testFlightInstallerStore = 'com.apple.testflight';

/// Opens the store page of the app.
///
/// On Android this opens the Play Store. On iOS this opens TestFlight when the app was installed through
/// TestFlight, and the App Store otherwise.
Future<void> openAppStore() async {
  final info = await PackageInfo.fromPlatform();
  if (Platform.isIOS) return _openIosStore(info);
  if (Platform.isAndroid) return _openPlayStore(info.packageName);
}

Future<void> _openPlayStore(String packageName) async {
  final opened = await launchUrlStringCatching('market://details?id=$packageName');
  if (opened) return;
  await launchUrlStringCatching('https://play.google.com/store/apps/details?id=$packageName');
}

Future<void> _openIosStore(PackageInfo info) async {
  if (info.installerStore == _testFlightInstallerStore) {
    await launchUrlStringCatching(_testFlightUrl);
    return;
  }
  final appStoreId = Environment.appStoreId;
  if (appStoreId.isEmpty) {
    Fimber.e('APP_STORE_ID is not set, unable to open the App Store');
    return;
  }
  await launchUrlStringCatching('itms-apps://apps.apple.com/app/id$appStoreId');
}
