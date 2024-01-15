import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/organization/organization_repository.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/wallet_card.dart';

part 'check_attributes_event.dart';
part 'check_attributes_state.dart';

class CheckAttributesBloc extends Bloc<CheckAttributesEvent, CheckAttributesState> {
  final OrganizationRepository _organizationRepository;
  final WalletCard card;
  final List<DataAttribute> attributes;

  CheckAttributesBloc(this._organizationRepository, {required this.attributes, required this.card})
      : super(CheckAttributesInitial(card: card, attributes: attributes)) {
    on<CheckAttributesLoadTriggered>(_onCheckAttributesLoadTriggered);
  }

  void _onCheckAttributesLoadTriggered(event, emit) async {
    try {
      final issuer = await _organizationRepository.findIssuer(card.docType);
      emit(CheckAttributesSuccess(card: card, attributes: attributes, cardIssuer: issuer!));
    } catch (ex) {
      Fimber.e('Issuer for ${card.docType} could not be found', ex: ex);
    }
  }
}
