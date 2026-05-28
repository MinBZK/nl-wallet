import 'dart:ui';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/help/help_category.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/help/get_help_categories_usecase.dart';

part 'help_overview_event.dart';
part 'help_overview_state.dart';

class HelpOverviewBloc extends Bloc<HelpOverviewEvent, HelpOverviewState> {
  final GetHelpCategoriesUseCase getHelpCategoriesUseCase;
  final Locale locale;

  HelpOverviewBloc(this.getHelpCategoriesUseCase, this.locale) : super(const HelpOverviewInitial()) {
    on<HelpOverviewLoadTriggered>(_onLoadTriggered);
  }

  Future<void> _onLoadTriggered(HelpOverviewLoadTriggered event, Emitter<HelpOverviewState> emit) async {
    emit(const HelpOverviewLoadInProgress());
    final result = await getHelpCategoriesUseCase.invoke(locale);
    await result.process(
      onSuccess: (categories) => emit(HelpOverviewLoadSuccess(categories)),
      onError: (error) => emit(HelpOverviewLoadFailure(error)),
    );
  }
}
