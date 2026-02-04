import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

part 'review_revocation_code_event.dart';
part 'review_revocation_code_state.dart';

class ReviewRevocationCodeBloc extends Bloc<ReviewRevocationCodeEvent, ReviewRevocationCodeState> {
  ReviewRevocationCodeBloc() : super(const ReviewRevocationCodeInitial()) {
    on<ReviewRevocationCodeRequested>(_onRevocationCodeRequested);
    on<ReviewRevocationCodeLoaded>(_onRevocationCodeLoaded);
    on<ReviewRevocationCodeRestartFlow>(_onRevocationCodeSavedByUser);
  }

  FutureOr<void> _onRevocationCodeRequested(
    ReviewRevocationCodeRequested event,
    Emitter<ReviewRevocationCodeState> emit,
  ) async => emit(const ReviewRevocationCodeProvidePin());

  FutureOr<void> _onRevocationCodeLoaded(
    ReviewRevocationCodeLoaded event,
    Emitter<ReviewRevocationCodeState> emit,
  ) async => emit(ReviewRevocationCodeSuccess(event.revocationCode));

  FutureOr<void> _onRevocationCodeSavedByUser(
    ReviewRevocationCodeRestartFlow event,
    Emitter<ReviewRevocationCodeState> emit,
  ) async => emit(const ReviewRevocationCodeInitial());
}
