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

  const SplashLoaded({required this.isRegistered, required this.hasPid}) : assert(!hasPid || isRegistered);

  @override
  List<Object> get props => [isRegistered, hasPid];
}
