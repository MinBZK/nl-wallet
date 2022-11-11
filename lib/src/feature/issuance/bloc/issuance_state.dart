part of 'issuance_bloc.dart';

abstract class IssuanceState extends Equatable {
  const IssuanceState();
}

class IssuanceInitial extends IssuanceState {
  @override
  List<Object> get props => [];
}
