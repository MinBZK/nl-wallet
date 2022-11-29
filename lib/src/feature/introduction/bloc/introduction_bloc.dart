import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'introduction_event.dart';
part 'introduction_state.dart';

class IntroductionBloc extends Bloc<IntroductionEvent, IntroductionState> {
  IntroductionBloc() : super(const IntroductionAppDisclaimer()) {
    on<IntroductionNextPressed>(_onIntroductionNextPressed);
    on<IntroductionBackPressed>(_onIntroductionBackPressed);
  }

  FutureOr<void> _onIntroductionNextPressed(event, emit) async {
    final state = this.state;
    if (state is IntroductionAppDisclaimer) {
      emit(const IntroductionAppIntroduction());
    } else if (state is IntroductionAppIntroduction) {
      emit(const IntroductionAppBenefits());
    } else if (state is IntroductionAppBenefits) {
      emit(IntroductionAppSecurity());
    }
  }

  FutureOr<void> _onIntroductionBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is IntroductionAppIntroduction) {
        emit(const IntroductionAppDisclaimer(afterBackPressed: true));
      } else if (state is IntroductionAppBenefits) {
        emit(const IntroductionAppIntroduction(afterBackPressed: true));
      } else if (state is IntroductionAppSecurity) {
        emit(const IntroductionAppBenefits(afterBackPressed: true));
      }
    }
  }
}
