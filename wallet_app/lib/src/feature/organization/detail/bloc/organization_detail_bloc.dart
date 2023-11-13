import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';

import '../../../../domain/model/organization.dart';
import '../../../../domain/usecase/history/has_previously_interacted_with_organization_usecase.dart';
import '../../../../domain/usecase/organization/get_organization_by_id_usecase.dart';

part 'organization_detail_event.dart';
part 'organization_detail_state.dart';

class OrganizationDetailBloc extends Bloc<OrganizationDetailEvent, OrganizationDetailState> {
  final GetOrganizationByIdUseCase _getOrganizationByIdUseCase;
  final HasPreviouslyInteractedWithOrganizationUseCase _hasPreviouslyInteractedWithOrganizationUseCase;

  OrganizationDetailBloc(this._getOrganizationByIdUseCase, this._hasPreviouslyInteractedWithOrganizationUseCase,
      {Organization? organization})
      : super(OrganizationDetailInitial()) {
    on<OrganizationLoadTriggered>(_onOrganizationLoadTriggered);
  }

  OrganizationDetailBloc.forOrganization(
    this._getOrganizationByIdUseCase,
    this._hasPreviouslyInteractedWithOrganizationUseCase, {
    required Organization organization,
    required bool isFirstInteractionWithOrganization,
  }) : super(OrganizationDetailSuccess(
          organization: organization,
          isFirstInteractionWithOrganization: isFirstInteractionWithOrganization,
        )) {
    on<OrganizationLoadTriggered>(_onOrganizationLoadTriggered);
  }

  void _onOrganizationLoadTriggered(event, emit) async {
    try {
      final organization = await _getOrganizationByIdUseCase.invoke(event.organizationId);
      var hasInteraction = await _hasPreviouslyInteractedWithOrganizationUseCase.invoke(event.organizationId);
      emit(
        OrganizationDetailSuccess(
          organization: organization!,
          isFirstInteractionWithOrganization: hasInteraction,
        ),
      );
    } catch (exception) {
      Fimber.e('Failed to fetch organization for ${event.organizationId}', ex: exception);
      emit(OrganizationDetailFailure(organizationId: event.organizationId));
    }
  }
}
