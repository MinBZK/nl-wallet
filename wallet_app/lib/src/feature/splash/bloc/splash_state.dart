part of 'splash_bloc.dart';

sealed class SplashState extends Equatable {
  const SplashState();
}

class SplashInitial extends SplashState {
  @override
  List<Object> get props => [];
}

class SplashLoaded extends SplashState {
  final bool isRegistered;
  final bool hasPid;

  const SplashLoaded({required this.isRegistered, required this.hasPid})
    : assert(!hasPid || isRegistered, 'The user should never have a pid but NOT be registered');

  @override
  List<Object> get props => [isRegistered, hasPid];
}
