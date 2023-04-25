class OrganizationDetailScreenArgument {
  static const _kTitleKey = 'title';
  static const _kOrganizationIdKey = 'organization_id';

  final String title;
  final String organizationId;

  const OrganizationDetailScreenArgument({
    required this.title,
    required this.organizationId,
  });

  Map<String, dynamic> toMap() {
    return {
      _kTitleKey: title,
      _kOrganizationIdKey: organizationId,
    };
  }

  static OrganizationDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return OrganizationDetailScreenArgument(
      title: map[_kTitleKey],
      organizationId: map[_kOrganizationIdKey],
    );
  }
}
