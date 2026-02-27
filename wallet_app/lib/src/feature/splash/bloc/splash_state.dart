part of 'splash_bloc.dart';

sealed class SplashState extends Equatable {
  const SplashState();
}

class SplashInitial extends SplashState {
  @override
  List<Object> get props => [];
}

class SplashLoaded extends SplashState {
  final PostSplashDestination destination;

  const SplashLoaded(this.destination);

  @override
  List<Object> get props => [destination];
}

enum PostSplashDestination { onboarding, revocationCode, pidRetrieval, transfer, pinRecovery, dashboard, blocked, none }
