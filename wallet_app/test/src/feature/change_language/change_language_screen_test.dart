import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/change_language/bloc/change_language_bloc.dart';
import 'package:wallet/src/feature/change_language/change_language_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockChangeLanguageBloc extends MockBloc<ChangeLanguageEvent, ChangeLanguageState> implements ChangeLanguageBloc {}

void main() {
  const mockLanguages = [
    Language('English', Locale('en')),
    Language('Dutch', Locale('nl')),
    Language('Spanish', Locale('es')),
  ];

  group('goldens', () {
    testGoldens('ChangeLanguageScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ChangeLanguageScreen().withState<ChangeLanguageBloc, ChangeLanguageState>(
          MockChangeLanguageBloc(),
          ChangeLanguageSuccess(availableLanguages: mockLanguages, selectedLocale: mockLanguages.first.locale),
        ),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ChangeLanguageScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ChangeLanguageScreen().withState<ChangeLanguageBloc, ChangeLanguageState>(
          MockChangeLanguageBloc(),
          ChangeLanguageSuccess(availableLanguages: mockLanguages, selectedLocale: mockLanguages.first.locale),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });

    testGoldens('Loading state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ChangeLanguageScreen().withState<ChangeLanguageBloc, ChangeLanguageState>(
          MockChangeLanguageBloc(),
          ChangeLanguageInitial(),
        ),
      );
      await screenMatchesGolden('loading');
    });
  });

  group('widgets', () {
    testWidgets('languages are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ChangeLanguageScreen().withState<ChangeLanguageBloc, ChangeLanguageState>(
          MockChangeLanguageBloc(),
          ChangeLanguageSuccess(availableLanguages: mockLanguages, selectedLocale: mockLanguages.first.locale),
        ),
      );
      await tester.pumpAndSettle();

      // Validate that the widget exists
      for (final language in mockLanguages) {
        expect(find.text(language.name, findRichText: true), findsOneWidget);
      }
    });
  });
}
