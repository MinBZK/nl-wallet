part of 'splash_bloc.dart';

abstract class SplashEvent extends Equatable {
  const SplashEvent();
}

class InitSplashEvent extends SplashEvent {
  const InitSplashEvent();

  @override
  List<Object?> get props => [];
}
