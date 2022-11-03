part of 'verification_bloc.dart';

abstract class VerificationState extends Equatable {
  const VerificationState();
}

class VerificationInitial extends VerificationState {
  @override
  List<Object> get props => [];
}

class VerificationLoadInProgress extends VerificationState {
  @override
  List<Object> get props => [];
}

class VerificationLoadFailure extends VerificationState {
  @override
  List<Object> get props => [];
}

class VerificationLoadSuccess extends VerificationState {
  final VerificationRequest request;

  const VerificationLoadSuccess({
    required this.request,
  });

  @override
  List<Object> get props => [request];
}
