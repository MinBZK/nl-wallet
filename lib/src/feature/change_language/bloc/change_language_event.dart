part of 'change_language_bloc.dart';

abstract class ChangeLanguageEvent extends Equatable {
  const ChangeLanguageEvent();
}

class ChangeLanguageLoadTriggered extends ChangeLanguageEvent {
  @override
  List<Object?> get props => [];
}

class ChangeLanguageLocaleSelected extends ChangeLanguageEvent {
  final Locale selectedLocale;

  const ChangeLanguageLocaleSelected(this.selectedLocale);

  @override
  List<Object?> get props => [selectedLocale];
}
