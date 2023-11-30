import '../../../../data/repository/organization/organization_repository.dart';

class OrganizationDetailScreenArgument {
  static const _kTitleKey = 'title';
  static const _kOrganizationIdKey = 'organization_id';
  static const _kOrganizationKey = 'organization';

  final String title;
  final String organizationId;
  final Organization? organization;

  const OrganizationDetailScreenArgument({
    required this.title,
    required this.organizationId,
    this.organization,
  });

  Map<String, dynamic> toMap() {
    return {
      _kTitleKey: title,
      _kOrganizationIdKey: organizationId,
      _kOrganizationKey: organization,
    };
  }

  static OrganizationDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return OrganizationDetailScreenArgument(
      title: map[_kTitleKey],
      organizationId: map[_kOrganizationIdKey],
      organization: map[_kOrganizationKey],
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is OrganizationDetailScreenArgument &&
          runtimeType == other.runtimeType &&
          title == other.title &&
          organizationId == other.organizationId &&
          organization == other.organization;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        title,
        organizationId,
        organization,
      );
}
