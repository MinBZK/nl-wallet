import 'dart:ui';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/language/language_repository.dart';

part 'change_language_event.dart';
part 'change_language_state.dart';

final Map<String, String> _kLanguageMap = {
  'nl': 'Nederlands',
  'en': 'English',
};

class ChangeLanguageBloc extends Bloc<ChangeLanguageEvent, ChangeLanguageState> {
  final LanguageRepository _languageRepository;
  final DefaultLocaleProvider _defaultLocaleProvider;

  ChangeLanguageBloc(this._languageRepository, this._defaultLocaleProvider) : super(ChangeLanguageInitial()) {
    on<ChangeLanguageLoadTriggered>(_onChangeLanguageLoadTriggered);
    on<ChangeLanguageLocaleSelected>(_onChangeLanguageLocaleSelected);
  }

  Future<void> _onChangeLanguageLocaleSelected(
    ChangeLanguageLocaleSelected event,
    Emitter<ChangeLanguageState> emit,
  ) async {
    final localState = state;
    if (localState is ChangeLanguageSuccess) {
      await _languageRepository.setPreferredLocale(event.selectedLocale);
      emit(localState.copyWith(selectedLocale: event.selectedLocale));
    }
  }

  Future<void> _onChangeLanguageLoadTriggered(
    ChangeLanguageLoadTriggered event,
    Emitter<ChangeLanguageState> emit,
  ) async {
    final locales = await _languageRepository.getAvailableLocales();
    final locale = await _languageRepository.preferredLocale.first ?? _defaultLocaleProvider();
    final languages = locales.map((locale) {
      return Language(_kLanguageMap[locale.languageCode] ?? locale.languageCode, locale);
    });
    emit(ChangeLanguageSuccess(selectedLocale: locale, availableLanguages: languages.toList()));
  }
}

typedef DefaultLocaleProvider = Locale Function();

class Language extends Equatable {
  final String name;
  final Locale locale;

  const Language(this.name, this.locale);

  @override
  List<Object?> get props => [name, locale];
}
