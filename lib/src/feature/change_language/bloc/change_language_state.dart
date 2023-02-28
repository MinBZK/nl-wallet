part of 'change_language_bloc.dart';

abstract class ChangeLanguageState extends Equatable {
  const ChangeLanguageState();
}

class ChangeLanguageInitial extends ChangeLanguageState {
  @override
  List<Object> get props => [];
}

class ChangeLanguageSuccess extends ChangeLanguageState {
  final Locale selectedLocale;
  final List<Language> availableLanguages;

  const ChangeLanguageSuccess({required this.selectedLocale, required this.availableLanguages});

  @override
  List<Object> get props => [selectedLocale, availableLanguages];

  ChangeLanguageSuccess copyWith({
    Locale? selectedLocale,
  }) {
    return ChangeLanguageSuccess(
      availableLanguages: availableLanguages,
      selectedLocale: selectedLocale ?? this.selectedLocale,
    );
  }
}
