import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/feature/common/sheet/error_details_sheet.dart';
import 'package:wallet/src/feature/common/widget/version/config_version_text.dart';
import 'package:wallet/src/feature/common/widget/version/os_version_text.dart';
import 'package:wallet/src/feature/common/widget/version/string_version_text.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';

const kGoldenSize = Size(350, 300);

void main() {
  late MockConfigurationRepository configurationRepository;

  setUp(() {
    provideDummy<Result<String>>(const Result.success('1.0'));
    configurationRepository = MockConfigurationRepository();
    when(configurationRepository.appConfiguration).thenAnswer(
      (_) => Stream.value(
        FlutterAppConfiguration(
          idleLockTimeout: Duration(),
          idleWarningTimeout: Duration(),
          backgroundLockTimeout: Duration(),
          version: 1337,
        ),
      ),
    );
  });

  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ErrorDetailsSheet()
              .withDependency<GetVersionStringUseCase>((c) => MockGetVersionStringUseCase())
              .withDependency<ConfigurationRepository>((c) => configurationRepository),
          surfaceSize: kGoldenSize,
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('error_details_sheet/light');
      },
    );

    testGoldens(
      'light - with application error',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ErrorDetailsSheet(
            error: GenericError(
              'Raw error message',
              sourceError: Exception('Some exception message'),
            ),
          )
              .withDependency<GetVersionStringUseCase>((c) => MockGetVersionStringUseCase())
              .withDependency<ConfigurationRepository>((c) => configurationRepository),
          surfaceSize: Size(350, 341),
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('error_details_sheet/application_error.light');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ErrorDetailsSheet()
              .withDependency<GetVersionStringUseCase>((c) => MockGetVersionStringUseCase())
              .withDependency<ConfigurationRepository>((c) => configurationRepository),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('error_details_sheet/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('version widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ErrorDetailsSheet()
            .withDependency<GetVersionStringUseCase>((c) => MockGetVersionStringUseCase())
            .withDependency<ConfigurationRepository>((c) => configurationRepository),
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
