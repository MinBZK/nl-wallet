part of 'review_revocation_code_bloc.dart';

abstract class ReviewRevocationCodeEvent extends Equatable {
  const ReviewRevocationCodeEvent();

  @override
  List<Object> get props => [];
}

class ReviewRevocationCodeRequested extends ReviewRevocationCodeEvent {
  const ReviewRevocationCodeRequested();
}

class ReviewRevocationCodeLoaded extends ReviewRevocationCodeEvent {
  final String revocationCode;

  const ReviewRevocationCodeLoaded(this.revocationCode);

  @override
  List<Object> get props => [revocationCode];
}

class ReviewRevocationCodeRestartFlow extends ReviewRevocationCodeEvent {
  const ReviewRevocationCodeRestartFlow();
}
