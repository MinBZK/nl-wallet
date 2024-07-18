import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/feature/common/sheet/error_details_sheet.dart';
import 'package:wallet/src/feature/common/widget/config_version_text.dart';
import 'package:wallet/src/feature/common/widget/os_version_text.dart';
import 'package:wallet/src/feature/common/widget/version_text.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  group('widgets', () {
    testWidgets('version widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ErrorDetailsSheet().withDependency<ConfigurationRepository>(
          (c) => MockConfigurationRepository(),
        ),
      );

      // Validate that the widget exists
      final versionFinder = find.byType(VersionText);
      final osVersionFinder = find.byType(OsVersionText);
      final configVersionFinder = find.byType(ConfigVersionText);
      expect(versionFinder, findsOneWidget);
      expect(osVersionFinder, findsOneWidget);
      expect(configVersionFinder, findsOneWidget);
    });
  });
}
