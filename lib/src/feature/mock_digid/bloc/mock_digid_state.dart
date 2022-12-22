part of 'mock_digid_bloc.dart';

abstract class MockDigidState extends Equatable {
  const MockDigidState();
}

class MockDigidInitial extends MockDigidState {
  @override
  List<Object> get props => [];
}

class MockDigidEnteringPin extends MockDigidState {
  final int enteredDigits;

  const MockDigidEnteringPin(this.enteredDigits);

  @override
  List<Object> get props => [enteredDigits];
}

class MockDigidConfirmApp extends MockDigidState {
  @override
  List<Object> get props => [];
}

class MockDigidLoadInProgress extends MockDigidState {
  final Duration mockDelay;

  const MockDigidLoadInProgress(this.mockDelay);

  @override
  List<Object> get props => [mockDelay];
}

class MockDigidLoggedIn extends MockDigidState {
  @override
  List<Object> get props => [];
}
