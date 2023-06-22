import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/organization/organization_logo.dart';

class OrganizationRow extends StatelessWidget {
  final String organizationName;
  final VoidCallback? onTap;
  final String? logoUrl;

  const OrganizationRow({
    required this.organizationName,
    this.onTap,
    this.logoUrl,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return InfoRow(
      icon: logoUrl != null ? null : Icons.apartment_outlined,
      leading: logoUrl != null ? OrganizationLogo(image: AssetImage(logoUrl!), size: 24) : null,
      title: context.l10n.organizationButtonLabel,
      subtitle: organizationName,
      onTap: onTap,
    );
  }
}
