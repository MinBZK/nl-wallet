import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/change_language/bloc/change_language_bloc.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockLanguageRepository languageRepository;
  late DefaultLocaleProvider defaultLocaleProvider;

  setUp(() {
    languageRepository = MockLanguageRepository();
    defaultLocaleProvider = () => const Locale('en');
  });

  blocTest(
    'verify initial state',
    build: () => ChangeLanguageBloc(languageRepository, defaultLocaleProvider),
    verify: (bloc) {
      expect(bloc.state, ChangeLanguageInitial());
    },
  );

  blocTest(
    'verify loaded state',
    build: () => ChangeLanguageBloc(languageRepository, defaultLocaleProvider),
    act: (bloc) => bloc.add(ChangeLanguageLoadTriggered()),
    setUp: () {
      when(
        languageRepository.getAvailableLocales(),
      ).thenAnswer((_) => Future.value([const Locale('nl'), const Locale('en')]));
      when(languageRepository.preferredLocale).thenAnswer((_) => Stream.value(null));
    },
    expect: () => [
      const ChangeLanguageSuccess(
        selectedLocale: Locale('en'),
        availableLanguages: [
          Language('Nederlands', Locale('nl')),
          Language('English', Locale('en')),
        ],
      ),
    ],
  );

  blocTest(
    'verify updated locale state',
    build: () => ChangeLanguageBloc(languageRepository, defaultLocaleProvider),
    act: (bloc) async {
      bloc.add(ChangeLanguageLoadTriggered());
      await Future.delayed(const Duration(milliseconds: 100));
      bloc.add(const ChangeLanguageLocaleSelected(Locale('nl')));
    },
    setUp: () {
      when(
        languageRepository.getAvailableLocales(),
      ).thenAnswer((_) => Future.value([const Locale('nl'), const Locale('en')]));
      when(languageRepository.preferredLocale).thenAnswer((_) => Stream.value(null));
    },
    expect: () => [
      const ChangeLanguageSuccess(
        selectedLocale: Locale('en'),
        availableLanguages: [
          Language('Nederlands', Locale('nl')),
          Language('English', Locale('en')),
        ],
      ),
      const ChangeLanguageSuccess(
        selectedLocale: Locale('nl'),
        availableLanguages: [
          Language('Nederlands', Locale('nl')),
          Language('English', Locale('en')),
        ],
      ),
    ],
  );
}
