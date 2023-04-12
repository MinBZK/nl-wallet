part of 'card_data_bloc.dart';

abstract class CardDataState extends Equatable {
  const CardDataState();
}

class CardDataInitial extends CardDataState {
  @override
  List<Object> get props => [];
}

class CardDataLoadInProgress extends CardDataState {
  const CardDataLoadInProgress();

  @override
  List<Object?> get props => [];
}

class CardDataLoadSuccess extends CardDataState {
  final List<DataAttribute> attributes;

  const CardDataLoadSuccess(this.attributes);

  @override
  List<Object> get props => [attributes];
}

class CardDataLoadFailure extends CardDataState {
  const CardDataLoadFailure();

  @override
  List<Object> get props => [];
}
