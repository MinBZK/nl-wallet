import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:wallet/src/feature/common/widget/version_text.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(150, 20);
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
          const VersionText(),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'version_text/light');
      },
    );
    testGoldens(
      'dark version text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const VersionText(),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'version_text/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('version and buildNr are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const VersionText(),
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
