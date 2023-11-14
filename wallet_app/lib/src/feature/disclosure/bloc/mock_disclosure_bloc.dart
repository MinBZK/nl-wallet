import 'package:fimber/fimber.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/usecase/card/log_card_interaction_usecase.dart';
import '../../../domain/usecase/disclosure/get_disclosure_request_usecase.dart';
import '../../../domain/usecase/history/has_previously_interacted_with_organization_usecase.dart';
import '../../../domain/usecase/wallet/get_requested_attributes_with_card_usecase.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';
import '../model/disclosure_flow.dart';
import 'disclosure_bloc.dart';

class MockDisclosureBloc extends DisclosureBloc {
  final GetDisclosureRequestUseCase getDisclosureRequestUseCase;
  final LogCardInteractionUseCase logCardInteractionUseCase;
  final GetRequestedAttributesWithCardUseCase getRequestedAttributesWithCardUseCase;
  final HasPreviouslyInteractedWithOrganizationUseCase hasPreviouslyInteractedWithOrganizationUseCase;

  MockDisclosureBloc(
    this.getDisclosureRequestUseCase,
    this.getRequestedAttributesWithCardUseCase,
    this.logCardInteractionUseCase,
    this.hasPreviouslyInteractedWithOrganizationUseCase,
  ) {
    on<DisclosureLoadRequested>(_onDisclosureLoadRequested);
    on<DisclosureOrganizationApproved>(_onDisclosureOrganizationApproved);
    on<DisclosureShareRequestedAttributesApproved>(_onDisclosureShareRequestedAttributesApproved);
    on<DisclosurePinConfirmed>(_onDisclosurePinConfirmed);
    on<DisclosureBackPressed>(_onDisclosureBackPressed);
    on<DisclosureStopRequested>(_onDisclosureStopRequested);
    on<DisclosureReportPressed>(_onDisclosureReportPressed);
  }

  void _onDisclosureLoadRequested(DisclosureLoadRequested event, emit) async {
    if (state is DisclosureInitial) {
      try {
        emit(DisclosureLoadInProgress());
        final request = await getDisclosureRequestUseCase.invoke(event.sessionId);
        final disclosureFlow = DisclosureFlow(
          id: request.id,
          organization: request.organization,
          hasPreviouslyInteractedWithOrganization:
              await hasPreviouslyInteractedWithOrganizationUseCase.invoke(request.organization.id),
          availableAttributes: await getRequestedAttributesWithCardUseCase.invoke(request.requestedAttributes),
          requestedAttributes: request.requestedAttributes,
          requestPurpose: request.requestPurpose.untranslated,
          policy: request.interactionPolicy,
        );
        emit(DisclosureCheckOrganization.fromFlow(disclosureFlow));
      } catch (ex, stack) {
        Fimber.e('Failed to load DisclosureRequest for id ${event.sessionId}', ex: ex, stacktrace: stack);
        emit(DisclosureGenericError());
      }
    }
  }

  void _onDisclosureOrganizationApproved(DisclosureOrganizationApproved event, emit) {
    final state = this.state;
    if (state is DisclosureCheckOrganization) {
      if (state.flow!.hasMissingAttributes) {
        emit(DisclosureMissingAttributes.fromFlow(state.flow!));
      } else {
        emit(DisclosureConfirmDataAttributes.fromFlow(state.flow!));
      }
    }
  }

  void _onDisclosureShareRequestedAttributesApproved(DisclosureShareRequestedAttributesApproved event, emit) {
    final state = this.state;
    if (state is DisclosureConfirmDataAttributes) emit(DisclosureConfirmPin.fromFlow(state.flow!));
  }

  void _onDisclosurePinConfirmed(DisclosurePinConfirmed event, emit) async {
    final state = this.state;
    if (state is DisclosureConfirmPin) {
      emit(DisclosureLoadInProgress());
      if (event.flow != null) _logCardInteraction(event.flow!, InteractionStatus.success);
      await Future.delayed(kDefaultMockDelay);
      emit(DisclosureSuccess.fromFlow(state.flow!));
    }
  }

  void _onDisclosureBackPressed(DisclosureBackPressed event, emit) {
    final state = this.state;
    if (state.canGoBack) {
      if (state is DisclosureConfirmDataAttributes) {
        emit(DisclosureCheckOrganization.fromFlow(state.flow!, afterBackPressed: true));
      } else if (state is DisclosureMissingAttributes) {
        emit(DisclosureMissingAttributes.fromFlow(state.flow!));
      } else if (state is DisclosureConfirmPin) {
        emit(DisclosureConfirmDataAttributes.fromFlow(state.flow!, afterBackPressed: true));
      }
    }
  }

  void _onDisclosureStopRequested(DisclosureStopRequested event, emit) async {
    if (event.flow != null) _logCardInteraction(event.flow!, InteractionStatus.rejected);
    emit(const DisclosureStopped());
  }

  void _onDisclosureReportPressed(DisclosureReportPressed event, emit) async {
    if (event.flow != null) _logCardInteraction(event.flow!, InteractionStatus.rejected);
    emit(const DisclosureLeftFeedback());
  }

  void _logCardInteraction(DisclosureFlow flow, InteractionStatus status) {
    logCardInteractionUseCase.invoke(
      status: status,
      policy: flow.policy,
      organization: flow.organization,
      resolvedAttributes: flow.resolvedAttributes,
      requestPurpose: flow.requestPurpose.l10nValueFromLocale('nl'),
    );
  }
}
