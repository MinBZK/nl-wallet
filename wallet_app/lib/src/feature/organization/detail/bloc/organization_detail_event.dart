part of 'organization_detail_bloc.dart';

abstract class OrganizationDetailEvent extends Equatable {
  const OrganizationDetailEvent();
}

class OrganizationProvided extends OrganizationDetailEvent {
  final Organization organization;
  final bool sharedDataWithOrganizationBefore;

  const OrganizationProvided({
    required this.organization,
    required this.sharedDataWithOrganizationBefore,
  });

  @override
  List<Object?> get props => [organization, sharedDataWithOrganizationBefore];
}
