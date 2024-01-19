import '../../../../data/repository/organization/organization_repository.dart';

class OrganizationDetailScreenArgument {
  static const _kOrganizationKey = 'organization';
  static const _kIsFirstInteractionKey = 'first_interaction';

  final Organization organization;
  final bool isFirstInteractionWithOrganization;

  const OrganizationDetailScreenArgument({
    required this.organization,
    required this.isFirstInteractionWithOrganization,
  });

  Map<String, dynamic> toMap() {
    return {
      _kOrganizationKey: organization,
      _kIsFirstInteractionKey: isFirstInteractionWithOrganization,
    };
  }

  static OrganizationDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return OrganizationDetailScreenArgument(
      organization: map[_kOrganizationKey],
      isFirstInteractionWithOrganization: map[_kIsFirstInteractionKey],
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is OrganizationDetailScreenArgument &&
          runtimeType == other.runtimeType &&
          isFirstInteractionWithOrganization == other.isFirstInteractionWithOrganization &&
          organization == other.organization;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        isFirstInteractionWithOrganization,
        organization,
      );
}
