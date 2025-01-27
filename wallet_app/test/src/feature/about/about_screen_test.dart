import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/feature/about/about_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  late GetVersionStringUseCase getVersionUsecase;

  setUp(() async {
    getVersionUsecase = MockGetVersionStringUseCase();
    when(getVersionUsecase.invoke()).thenAnswer((_) async => '1.2.3 (123)');
  });

  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: const AboutScreen(),
          name: 'about',
        );
    }

    testGoldens('about light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(
          providers: [
            RepositoryProvider<GetVersionStringUseCase>(create: (c) => getVersionUsecase),
            RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('about dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
          providers: [
            RepositoryProvider<GetVersionStringUseCase>(create: (c) => getVersionUsecase),
            RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'dark');
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
