part of 'revocation_code_bloc.dart';

sealed class RevocationCodeState extends Equatable {
  const RevocationCodeState();

  @override
  List<Object?> get props => [];
}

class RevocationCodeInitial extends RevocationCodeState {
  final String? revocationCode;

  const RevocationCodeInitial({this.revocationCode});

  @override
  List<Object?> get props => [revocationCode];
}

class RevocationCodeLoadSuccess extends RevocationCodeState {
  final String revocationCode;

  const RevocationCodeLoadSuccess(this.revocationCode);
}

class RevocationCodeSaveSuccess extends RevocationCodeState {
  final String revocationCode;

  const RevocationCodeSaveSuccess(this.revocationCode);
}
