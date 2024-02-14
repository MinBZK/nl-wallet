import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';

import '../../../../domain/model/organization.dart';

part 'organization_detail_event.dart';
part 'organization_detail_state.dart';

class OrganizationDetailBloc extends Bloc<OrganizationDetailEvent, OrganizationDetailState> {
  OrganizationDetailBloc() : super(OrganizationDetailInitial()) {
    on<OrganizationProvided>(_onOrganizationProvided);
  }

  void _onOrganizationProvided(OrganizationProvided event, emit) async {
    emit(
      OrganizationDetailSuccess(
        organization: event.organization,
        sharedDataWithOrganizationBefore: event.sharedDataWithOrganizationBefore,
      ),
    );
  }
}
