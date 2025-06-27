import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';

part 'check_attributes_event.dart';
part 'check_attributes_state.dart';

class CheckAttributesBloc extends Bloc<CheckAttributesEvent, CheckAttributesState> {
  final List<WalletCard> cards;

  CheckAttributesBloc({required this.cards})
      : assert(cards.isNotEmpty, 'provide at least one card'),
        super(
          cards.length == 1
              ? CheckAttributesSuccess(card: cards.first, attributes: cards.first.attributes)
              : CheckAttributesInitial(),
        ) {
    on<CheckAttributesCardSelected>(_onCardSelected);
  }

  factory CheckAttributesBloc.forCard(WalletCard card, {List<DataAttribute>? attributes}) =>
      CheckAttributesBloc(cards: [card.copyWith(attributes: attributes)]);

  Future<void> _onCardSelected(CheckAttributesCardSelected event, Emitter<CheckAttributesState> emit) async {
    emit(
      CheckAttributesSuccess(
        card: event.card,
        attributes: event.card.attributes,
        alternatives: [...cards]..remove(event.card),
      ),
    );
  }
}
