import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/feature/about/about_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  late GetVersionStringUseCase getVersionUsecase;

  setUp(() async {
    provideDummy<Result<String>>(const Result.success('1.0'));
    getVersionUsecase = MockGetVersionStringUseCase();
    when(getVersionUsecase.invoke()).thenAnswer((_) async => const Result.success('1.2.3 (123)'));
  });

  group('goldens', () {
    testGoldens('about light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen(),
        providers: [
          RepositoryProvider<GetVersionStringUseCase>(create: (c) => getVersionUsecase),
          RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('about light - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen(),
        providers: [
          RepositoryProvider<GetVersionStringUseCase>(create: (c) => getVersionUsecase),
          RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
        ],
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('light.landscape');
    });

    testGoldens('about dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen(),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<GetVersionStringUseCase>(create: (c) => getVersionUsecase),
          RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
        ],
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('about the app title is visible', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen()
            .withDependency<GetVersionStringUseCase>((context) => getVersionUsecase)
            .withDependency<ConfigurationRepository>((context) => MockConfigurationRepository()),
      );

      // Validate that the widget exists
      final widgetFinder = find.text(l10n.aboutScreenTitle);
      expect(widgetFinder, findsNWidgets(2));
    });
  });
}
