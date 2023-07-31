import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:wallet/src/feature/common/sheet/help_sheet.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(350, 360);

  setUp(() {
    PackageInfo.setMockInitialValues(
      appName: 'appName',
      packageName: 'packageName',
      version: '1.0.0',
      buildNumber: '1',
      buildSignature: 'signature',
    );
  });

  group('goldens', () {
    testGoldens(
      'light help sheet',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(const HelpSheet(), surfaceSize: kGoldenSize);
        await screenMatchesGolden(tester, 'help_sheet/light');
      },
    );
    testGoldens(
      'dark help sheet with error and support code',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const HelpSheet(
            errorCode: '1.2.3.4',
            supportCode: 'abc123',
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'help_sheet/error_and_support.dark');
      },
    );
  });
}
