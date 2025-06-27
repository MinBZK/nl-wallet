part of 'check_attributes_bloc.dart';

abstract class CheckAttributesEvent extends Equatable {
  const CheckAttributesEvent();
}

class CheckAttributesCardSelected extends CheckAttributesEvent {
  final WalletCard card;

  const CheckAttributesCardSelected({required this.card});

  @override
  List<Object?> get props => [card];
}
