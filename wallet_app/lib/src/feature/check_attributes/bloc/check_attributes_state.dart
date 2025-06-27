part of 'check_attributes_bloc.dart';

sealed class CheckAttributesState extends Equatable {
  const CheckAttributesState();
}

class CheckAttributesInitial extends CheckAttributesState {
  @override
  List<Object?> get props => [];
}

class CheckAttributesSuccess extends CheckAttributesState {
  final WalletCard card;
  final List<DataAttribute> attributes;
  final List<WalletCard>? alternatives;

  bool get showChangeCardCta => alternatives?.isNotEmpty ?? false;

  const CheckAttributesSuccess({
    required this.card,
    required this.attributes,
    this.alternatives,
  });

  @override
  List<Object?> get props => [card, attributes, alternatives];
}
