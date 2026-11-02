import 'dart:ui';

import 'package:flutter/services.dart';
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
    getVersionUsecase = MockGetVersionStringUseCase();
    when(getVersionUsecase.invoke()).thenAnswer((_) async => const Result.success('1.2.3 (123)'));
  });

  group('goldens', () {
    testGoldens('ltc28 about light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen(),
        providers: [
          RepositoryProvider<GetVersionStringUseCase>(create: (c) => getVersionUsecase),
          RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ltc28 about light - landscape', (tester) async {
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

    testGoldens('ltc28 about dark', (tester) async {
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
    testWidgets('ltc28 about the app title is visible', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen()
            .withDependency<GetVersionStringUseCase>((context) => getVersionUsecase)
            .withDependency<ConfigurationRepository>((context) => MockConfigurationRepository()),
      );

      // Validate that the widget exists
      final widgetFinder = find.text(l10n.aboutScreenTitle);
      expect(widgetFinder, findsOneWidget);
    });

    testWidgets('ltc28 only the url in the description is exposed as a link', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen()
            .withDependency<GetVersionStringUseCase>((context) => getVersionUsecase)
            .withDependency<ConfigurationRepository>((context) => Mocks.create()),
      );
      await tester.pumpAndSettle();

      final url = l10n.aboutScreenDescriptionLink;
      final linkFinder = find.semantics.byFlag(SemanticsFlag.isLink);
      expect(linkFinder, findsOne);
      final link = linkFinder.evaluate().single;
      expect(link, isSemantics(label: url, isLink: true, isButton: false, hasTapAction: true));
      expect(link.getSemanticsData().linkUrl, Uri.parse('https://$url'));

      final textBeforeLink = l10n.aboutScreenDescription(url).split('\n\n').last.split(url).first;
      final textFinder = find.semantics.byLabel(textBeforeLink);
      expect(textFinder, findsOne);
      expect(textFinder.evaluate().single, isSemantics(isButton: false, hasTapAction: false));
      semanticsHandle.dispose();
    });

    testWidgets('ltc28 app version and config are separate semantics nodes', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen()
            .withDependency<GetVersionStringUseCase>((context) => getVersionUsecase)
            .withDependency<ConfigurationRepository>((context) => Mocks.create()),
      );
      await tester.pumpAndSettle();

      final versionSemantics = tester.getSemantics(find.textContaining(l10n.generalVersionText, findRichText: true));
      final configSemantics = tester.getSemantics(
        find.textContaining(l10n.generalConfigVersionText, findRichText: true),
      );
      expect(versionSemantics.label, isNot(contains(l10n.generalConfigVersionText)));
      expect(configSemantics.label, isNot(contains(l10n.generalVersionText)));
    });

    testWidgets('ltc28 activating the link opens the url', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      final launchedUrls = <String>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('plugins.flutter.io/url_launcher'),
        (MethodCall methodCall) async {
          launchedUrls.add(methodCall.arguments['url'] as String);
          return true;
        },
      );
      addTearDown(
        () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
          const MethodChannel('plugins.flutter.io/url_launcher'),
          null,
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        const AboutScreen()
            .withDependency<GetVersionStringUseCase>((context) => getVersionUsecase)
            .withDependency<ConfigurationRepository>((context) => Mocks.create()),
      );
      await tester.pumpAndSettle();

      tester.semantics.tap(find.semantics.byFlag(SemanticsFlag.isLink));
      await tester.pumpAndSettle();

      expect(launchedUrls, ['https://${l10n.aboutScreenDescriptionLink}']);
      semanticsHandle.dispose();
    });
  });
}
