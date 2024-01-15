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
  final Organization cardIssuer;

  const CheckAttributesSuccess({
    required super.card,
    required super.attributes,
    required this.cardIssuer,
  });

  @override
  List<Object> get props => [card, attributes, cardIssuer];
}
