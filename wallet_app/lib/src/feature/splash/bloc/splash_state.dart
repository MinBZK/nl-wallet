part of 'splash_bloc.dart';

abstract class SplashState extends Equatable {
  const SplashState();
}

class SplashInitial extends SplashState {
  @override
  List<Object> get props => [];
}

class SplashLoaded extends SplashState {
  final bool isRegistered;
  final bool hasPid;

  const SplashLoaded({required this.isRegistered, required this.hasPid});

  @override
  List<Object> get props => [isRegistered, hasPid];
}
