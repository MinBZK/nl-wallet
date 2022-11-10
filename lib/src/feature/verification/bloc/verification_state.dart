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
  final VerificationResult status;

  const VerificationLoadSuccess({
    required this.request,
    this.status = VerificationResult.pendingUser,
  });

  @override
  List<Object> get props => [request, status];

  VerificationLoadSuccess copyWith({VerificationResult? status}) {
    return VerificationLoadSuccess(
      request: request,
      status: status ?? this.status,
    );
  }
}

enum VerificationResult { pendingUser, loading, approved, denied }
