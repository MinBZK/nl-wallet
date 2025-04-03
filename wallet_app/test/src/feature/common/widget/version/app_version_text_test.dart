import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:wallet/src/feature/common/widget/version/app_version_text.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(151, 20);
  const appVersion = '1.2.3';
  const buildNumber = '1337';

  setUp(() {
    PackageInfo.setMockInitialValues(
      appName: 'appName',
      packageName: 'packageName',
      version: appVersion,
      buildNumber: buildNumber,
      buildSignature: 'buildSignature',
    );
  });

  group('goldens', () {
    testGoldens(
      'light version text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const AppVersionText(),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('version_text/light');
      },
    );
    testGoldens(
      'dark version text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const AppVersionText(),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('version_text/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('version and buildNr are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppVersionText(),
      );
      await tester.pumpAndSettle();

      // Validate that the widget exists
      final versionFinder = find.textContaining(appVersion);
      final buildNrFinder = find.textContaining(buildNumber);
      expect(versionFinder, findsOneWidget);
      expect(buildNrFinder, findsOneWidget);
    });
  });
}
