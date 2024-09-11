import 'dart:async';
import 'dart:ui';

import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/language/language_repository.dart';

// Provides the user's preferred Locale, or 'null' if none is selected.
class PreferredLocaleCubit extends Cubit<Locale?> {
  final LanguageRepository languageRepository;

  StreamSubscription? _localeSubscription;

  PreferredLocaleCubit(this.languageRepository) : super(null) {
    _localeSubscription = languageRepository.preferredLocale.listen(emit);
  }

  @override
  Future<void> close() async {
    await _localeSubscription?.cancel();
    return super.close();
  }
}
