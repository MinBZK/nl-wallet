// coverage:ignore-file
import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../wallet_constants.dart';

part 'mock_digid_event.dart';
part 'mock_digid_state.dart';

class MockDigidBloc extends Bloc<MockDigidEvent, MockDigidState> {
  MockDigidBloc() : super(MockDigidInitial()) {
    on<MockDigidSplashDismissed>(_onSplashDismissed);
    on<MockDigidPinKeyPressed>(_onPinKeyPressed);
    on<MockDigidPinBackspacePressed>(_onPinBackspacePressed);
    on<MockDigidConfirmPressed>(_onConfirmPressed);
    on<MockDigidDeclinePressed>(_onDeclinePressed);

    //Dismiss digid splash after 2 seconds.
    Future.delayed(kDefaultDigidMockDelay).then((_) {
      if (!isClosed) add(MockDigidSplashDismissed());
    });
  }

  FutureOr<void> _onSplashDismissed(MockDigidSplashDismissed event, Emitter<MockDigidState> emit) async {
    emit(const MockDigidEnteringPin(0));
  }

  FutureOr<void> _onPinKeyPressed(MockDigidPinKeyPressed event, Emitter<MockDigidState> emit) async {
    final state = this.state;
    if (state is MockDigidEnteringPin && state.enteredDigits < 4) {
      emit(MockDigidEnteringPin(state.enteredDigits + 1));
    } else {
      emit(const MockDigidEnteringPin(5));
      await Future.delayed(const Duration(milliseconds: 150));
      emit(MockDigidConfirmApp());
    }
  }

  FutureOr<void> _onPinBackspacePressed(MockDigidPinBackspacePressed event, Emitter<MockDigidState> emit) async {
    final state = this.state;
    if (state is MockDigidEnteringPin && state.enteredDigits > 0) {
      emit(MockDigidEnteringPin(state.enteredDigits - 1));
    }
  }

  FutureOr<void> _onConfirmPressed(MockDigidConfirmPressed event, Emitter<MockDigidState> emit) async {
    emit(const MockDigidLoadInProgress(kDefaultDigidMockDelay));
    await Future.delayed(kDefaultDigidMockDelay);
    emit(MockDigidLoggedIn());
  }

  FutureOr<void> _onDeclinePressed(MockDigidDeclinePressed event, Emitter<MockDigidState> emit) async {
    emit(MockDigidRejected());
  }
}
