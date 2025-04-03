import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/feature/common/sheet/help_sheet.dart';
import 'package:wallet/src/feature/common/widget/version/app_version_text.dart';
import 'package:wallet/src/feature/common/widget/version/config_version_text.dart';
import 'package:wallet/src/feature/common/widget/version/os_version_text.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(350, 444);

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
        await tester.pumpWidgetWithAppWrapper(
          const HelpSheet(),
          surfaceSize: kGoldenSize,
          providers: [
            RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
          ],
        );
        await screenMatchesGolden('help_sheet/light');
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
          providers: [
            RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
          ],
        );
        await screenMatchesGolden('help_sheet/error_and_support.dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('errorCode, supportCode and version widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HelpSheet(
          errorCode: 'ERROR_CODE',
          supportCode: 'SUPPORT_CODE',
        ).withDependency<ConfigurationRepository>((_) => MockConfigurationRepository()),
      );

      // Validate that the widget exists
      final errorCodeFinder = find.textContaining('ERROR_CODE');
      final supportCodeFinder = find.textContaining('SUPPORT_CODE');
      expect(errorCodeFinder, findsOneWidget);
      expect(supportCodeFinder, findsOneWidget);

      // Validate version widgets
      expect(find.byType(AppVersionText), findsOneWidget);
      expect(find.byType(OsVersionText), findsOneWidget);
      expect(find.byType(ConfigVersionText), findsOneWidget);
    });
  });
}
