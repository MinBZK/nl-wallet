import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/wallet_card.dart';

part 'check_attributes_event.dart';
part 'check_attributes_state.dart';

class CheckAttributesBloc extends Bloc<CheckAttributesEvent, CheckAttributesState> {
  final WalletCard card;
  final List<DataAttribute> attributes;

  CheckAttributesBloc({required this.attributes, required this.card})
      : super(CheckAttributesInitial(card: card, attributes: attributes)) {
    on<CheckAttributesLoadTriggered>(_onCheckAttributesLoadTriggered);
  }

  Future<void> _onCheckAttributesLoadTriggered(event, emit) async {
    try {
      emit(CheckAttributesSuccess(card: card, attributes: attributes));
    } catch (ex) {
      Fimber.e('Issuer for ${card.docType} could not be found', ex: ex);
    }
  }
}
