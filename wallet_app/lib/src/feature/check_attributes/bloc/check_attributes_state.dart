part of 'check_attributes_bloc.dart';

sealed class CheckAttributesState extends Equatable {
  final WalletCard card;
  final List<DataAttribute> attributes;

  const CheckAttributesState({required this.card, required this.attributes});
}

class CheckAttributesInitial extends CheckAttributesState {
  const CheckAttributesInitial({required super.card, required super.attributes});

  @override
  List<Object> get props => [card, attributes];
}

class CheckAttributesSuccess extends CheckAttributesState {
  const CheckAttributesSuccess({
    required super.card,
    required super.attributes,
  });

  @override
  List<Object> get props => [card, attributes];
}
