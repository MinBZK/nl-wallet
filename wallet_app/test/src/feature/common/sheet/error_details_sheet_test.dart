import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/feature/common/sheet/error_details_sheet.dart';
import 'package:wallet/src/feature/common/widget/version/config_version_text.dart';
import 'package:wallet/src/feature/common/widget/version/os_version_text.dart';
import 'package:wallet/src/feature/common/widget/version/string_version_text.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  group('widgets', () {
    testWidgets('version widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ErrorDetailsSheet()
            .withDependency<GetVersionStringUseCase>((c) => MockGetVersionStringUseCase())
            .withDependency<ConfigurationRepository>((c) => MockConfigurationRepository()),
      );

      // Validate that the widget exists
      final stringVersionFinder = find.byType(StringVersionText);
      final osVersionFinder = find.byType(OsVersionText);
      final configVersionFinder = find.byType(ConfigVersionText);
      expect(stringVersionFinder, findsOneWidget);
      expect(osVersionFinder, findsOneWidget);
      expect(configVersionFinder, findsOneWidget);
    });
  });
}
