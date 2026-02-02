part of 'review_revocation_code_bloc.dart';

sealed class ReviewRevocationCodeState extends Equatable {
  const ReviewRevocationCodeState();

  @override
  List<Object?> get props => [];
}

class ReviewRevocationCodeInitial extends ReviewRevocationCodeState {
  const ReviewRevocationCodeInitial();
}

class ReviewRevocationCodeProvidePin extends ReviewRevocationCodeState {
  const ReviewRevocationCodeProvidePin();
}

class ReviewRevocationCodeSuccess extends ReviewRevocationCodeState {
  final String revocationCode;

  const ReviewRevocationCodeSuccess(this.revocationCode);

  @override
  List<Object?> get props => [revocationCode];
}
