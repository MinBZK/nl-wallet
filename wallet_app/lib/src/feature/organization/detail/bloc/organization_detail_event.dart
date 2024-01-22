part of 'organization_detail_bloc.dart';

abstract class OrganizationDetailEvent extends Equatable {
  const OrganizationDetailEvent();
}

class OrganizationProvided extends OrganizationDetailEvent {
  final Organization organization;
  final bool isFirstInteractionWithOrganization;

  const OrganizationProvided({
    required this.organization,
    required this.isFirstInteractionWithOrganization,
  });

  @override
  List<Object?> get props => [organization, isFirstInteractionWithOrganization];
}
