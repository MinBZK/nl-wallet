import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/common/widget/button/icon/close_icon_button.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/tertiary_button.dart';
import 'package:wallet/src/feature/error/error_page.dart';
import 'package:wallet/src/feature/error/error_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ErrorScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ErrorScreen(
          title: 'Headline',
          description: 'Description',
          primaryButton: PrimaryButton(
            text: const Text('Primary'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('Secondary'),
            onPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('screen/light');
    });

    testGoldens('ErrorScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ErrorScreen(
          title: 'Headline',
          description: 'Description',
          primaryButton: PrimaryButton(
            text: const Text('Primary'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('Secondary'),
            onPressed: () {},
          ),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('screen/dark');
    });

    testGoldens('ErrorScreen.showGeneric()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorScreen.fromError(context, const GenericError('test', sourceError: 'test')),
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('screen/generic.light');
    });

    testGoldens('ErrorScreen.network() (has internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) =>
              ErrorScreen.fromError(context, const NetworkError(hasInternet: true, sourceError: 'test')),
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('screen/network.light');
    });

    testGoldens('ErrorScreen.network() (no internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) =>
              ErrorScreen.fromError(context, const NetworkError(hasInternet: false, sourceError: 'test')),
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('screen/network.no_internet.light');
    });

    testGoldens('ErrorScreen.deviceIncompatible()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorScreen.fromError(context, const HardwareUnsupportedError(sourceError: 'test')),
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('screen/device_incompatible.light');
    });

    testGoldens('ErrorScreen.sessionExpired()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => ErrorScreen.fromError(
            context,
            const SessionError(state: .expired, sourceError: 'test'),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('screen/session_expired.light');
    });
  });

  group('widgets', () {
    testWidgets('ErrorScreen renders expected widgets', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ErrorScreen(
          description: 'D',
          title: 'H',
          primaryButton: PrimaryButton(
            text: const Text('P'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('S'),
            onPressed: () {},
          ),
        ),
      );

      // Setup finders
      final descriptionFinder = find.text('D', findRichText: true);
      final headlineFinder = find.text('H', findRichText: true);
      final primaryActionFinder = find.text('P', findRichText: true);
      final secondaryActionFinder = find.text('S', findRichText: true);

      // Verify all expected widgets show up once
      expect(descriptionFinder, findsOneWidget);
      expect(headlineFinder, findsOneWidget);
      expect(primaryActionFinder, findsOneWidget);
      expect(secondaryActionFinder, findsOneWidget);
    });

    testWidgets('No Internet ErrorScreen renders expected widgets', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ErrorScreen.fromError(context, const NetworkError(hasInternet: false, sourceError: ''));
          },
        ),
      );
      await tester.pumpAndSettle();

      // Setup finders
      final AppLocalizations locale = await TestUtils.englishLocalizations;
      final descriptionFinder = find.text(locale.errorScreenNoInternetDescription, findRichText: true);
      final headlineFinder = find.text(locale.errorScreenNoInternetHeadline, findRichText: true);
      final primaryActionFinder = find.text(locale.generalRetry, findRichText: true);
      final secondaryActionFinder = find.text(locale.generalShowDetailsCta, findRichText: true);

      // Verify all expected widgets show up once
      expect(descriptionFinder, findsOneWidget);
      expect(headlineFinder, findsOneWidget);
      expect(primaryActionFinder, findsOneWidget);
      expect(secondaryActionFinder, findsOneWidget);
    });
  });

  testWidgets('Generic ErrorScreen renders expected widgets', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Builder(
        builder: (context) => ErrorScreen.fromError(context, const GenericError('', sourceError: '')),
      ),
    );
    await tester.pumpAndSettle();

    // Setup finders
    final AppLocalizations locale = await TestUtils.englishLocalizations;
    final descriptionFinder = find.text(locale.errorScreenGenericDescription, findRichText: true);
    final headlineFinder = find.text(locale.errorScreenGenericHeadline, findRichText: true);
    final primaryActionFinder = find.text(locale.generalRetry, findRichText: true);
    final secondaryActionFinder = find.text(locale.generalShowDetailsCta, findRichText: true);

    // Verify all expected widgets show up once
    expect(descriptionFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(primaryActionFinder, findsOneWidget);
    expect(secondaryActionFinder, findsOneWidget);
  });

  testWidgets('Device incompatible ErrorScreen renders expected widgets', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Builder(
        builder: (context) => ErrorScreen.fromError(context, const HardwareUnsupportedError(sourceError: '')),
      ),
    );
    await tester.pumpAndSettle();

    // Setup finders
    final AppLocalizations locale = await TestUtils.englishLocalizations;
    final descriptionFinder = find.text(locale.errorScreenDeviceIncompatibleDescription, findRichText: true);
    final headlineFinder = find.text(locale.errorScreenDeviceIncompatibleHeadline, findRichText: true);
    final secondaryActionFinder = find.text(locale.generalShowDetailsCta, findRichText: true);

    // Verify all expected widgets show up once
    expect(descriptionFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(secondaryActionFinder, findsOneWidget);
  });

  testWidgets('Network ErrorScreen renders expected widgets', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Builder(
        builder: (context) => ErrorScreen.fromError(
          context,
          const NetworkError(hasInternet: true, sourceError: ''),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Setup finders
    final AppLocalizations locale = await TestUtils.englishLocalizations;
    final descriptionFinder = find.text(locale.errorScreenServerDescription, findRichText: true);
    final headlineFinder = find.text(locale.errorScreenServerHeadline, findRichText: true);
    final primaryActionFinder = find.text(locale.generalRetry, findRichText: true);
    final secondaryActionFinder = find.text(locale.generalShowDetailsCta, findRichText: true);

    // Verify all expected widgets show up once
    expect(descriptionFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(secondaryActionFinder, findsOneWidget);
    expect(primaryActionFinder, findsOneWidget);
  });

  testWidgets('Session expired ErrorScreen renders expected widgets', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Builder(
        builder: (context) => ErrorScreen.fromError(context, const SessionError(state: .expired, sourceError: '')),
      ),
    );
    await tester.pumpAndSettle();
    // Setup finders
    final AppLocalizations locale = await TestUtils.englishLocalizations;
    final descriptionFinder = find.text(locale.errorScreenSessionExpiredDescription, findRichText: true);
    final headlineFinder = find.text(locale.errorScreenSessionExpiredHeadline, findRichText: true);
    final primaryActionFinder = find.text(locale.generalRetry, findRichText: true);
    final secondaryActionFinder = find.text(locale.generalShowDetailsCta, findRichText: true);

    // Verify all expected widgets show up once
    expect(descriptionFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(primaryActionFinder, findsOneWidget);
    expect(secondaryActionFinder, findsOneWidget);
  });

  /// ErrorScreen.fromError relies on ErrorPage.fromError which is tested thoroughly in error_page_test.dart.
  /// Add some basic tests here to verify error mapping and provided CtaStyle is respected.
  group('fromError', () {
    testWidgets('maps GenericError to ErrorScreen with expected properties', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final screen = ErrorScreen.fromError(
              context,
              const GenericError('msg', sourceError: 'error'),
              style: ErrorCtaStyle.retry,
            );
            final expectedPage = ErrorPage.generic(context, style: ErrorCtaStyle.retry);
            expect(screen.title, expectedPage.title);
            expect(screen.description, expectedPage.description);
            expect(screen.illustration, expectedPage.illustration);
            expect(screen.actions, isEmpty);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('maps NetworkError(hasInternet: true) to ErrorPage.server', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final screen = ErrorScreen.fromError(
              context,
              const NetworkError(hasInternet: true, sourceError: 'error'),
            );
            final expectedPage = ErrorPage.server(context, style: ErrorCtaStyle.retry);
            expect(screen.title, expectedPage.title);
            expect(screen.description, expectedPage.description);
            return const SizedBox.shrink();
          },
        ),
      );
    });

    testWidgets('applies ErrorCtaStyle.close and provides CloseIconButton', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final screen = ErrorScreen.fromError(
              context,
              const GenericError('msg', sourceError: 'error'),
              style: ErrorCtaStyle.close,
            );
            expect(screen.actions, hasLength(1));
            expect(screen.actions.first, isA<CloseIconButton>());
            return const SizedBox.shrink();
          },
        ),
      );
    });
  });
}
