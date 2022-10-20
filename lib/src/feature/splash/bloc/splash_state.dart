part of 'splash_bloc.dart';

abstract class SplashState extends Equatable {
  const SplashState();
}

class SplashInitial extends SplashState {
  @override
  List<Object> get props => [];
}

class SplashLoaded extends SplashState {
  final bool isInitialized;

  const SplashLoaded(this.isInitialized);

  @override
  List<Object> get props => [isInitialized];
}
