import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/tertiary_button.dart';
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
      await screenMatchesGolden('light');
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
      await screenMatchesGolden('dark');
    });

    testGoldens('ErrorScreen.showGeneric()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ErrorScreen.generic(context);
          },
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('generic.light');
    });

    testGoldens('ErrorScreen.network() (has internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ErrorScreen.network(
              context,
              error: const NetworkError(hasInternet: true, sourceError: 'test'),
            );
          },
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('network.light');
    });

    testGoldens('ErrorScreen.network() (no internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ErrorScreen.network(
              context,
              error: const NetworkError(hasInternet: false, sourceError: 'test'),
            );
          },
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('network.no_internet.light');
    });

    testGoldens('ErrorScreen.deviceIncompatible()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ErrorScreen.deviceIncompatible(context);
          },
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('device_incompatible.light');
    });

    testGoldens('ErrorScreen.sessionExpired()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ErrorScreen.sessionExpired(context);
          },
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('session_expired.light');
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
            return ErrorScreen.noInternet(context);
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
        builder: (context) {
          return ErrorScreen.generic(context);
        },
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
        builder: (context) {
          return ErrorScreen.deviceIncompatible(context);
        },
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
        builder: (context) => ErrorScreen.network(
          context,
          error: const NetworkError(hasInternet: true, sourceError: ''),
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
        builder: (context) {
          return ErrorScreen.sessionExpired(context);
        },
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
}
