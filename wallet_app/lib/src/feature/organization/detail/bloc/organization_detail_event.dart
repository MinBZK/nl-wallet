part of 'organization_detail_bloc.dart';

abstract class OrganizationDetailEvent extends Equatable {
  const OrganizationDetailEvent();
}

class OrganizationLoadTriggered extends OrganizationDetailEvent {
  final String organizationId;

  const OrganizationLoadTriggered({required this.organizationId});

  @override
  List<Object?> get props => [organizationId];
}
