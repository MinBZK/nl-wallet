import 'dart:ui';

import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/language/language_repository.dart';

// Provides the user's preferred Locale, or 'null' if none is selected.
class PreferredLocaleCubit extends Cubit<Locale?> {
  final LanguageRepository languageRepository;

  PreferredLocaleCubit(this.languageRepository) : super(null) {
    languageRepository.preferredLocale.listen((event) => emit(event));
  }
}
