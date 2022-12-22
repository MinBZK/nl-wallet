part of 'mock_digid_bloc.dart';

abstract class MockDigidEvent extends Equatable {
  const MockDigidEvent();
}

class MockDigidSplashDismissed extends MockDigidEvent {
  @override
  List<Object?> get props => [];
}

class MockDigidPinKeyPressed extends MockDigidEvent {
  @override
  List<Object?> get props => [];
}

class MockDigidPinBackspacePressed extends MockDigidEvent {
  @override
  List<Object?> get props => [];
}

class MockDigidConfirmPressed extends MockDigidEvent {
  @override
  List<Object?> get props => [];
}
