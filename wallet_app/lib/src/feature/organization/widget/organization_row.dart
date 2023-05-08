import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

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
      title: AppLocalizations.of(context).organizationButtonLabel,
      subtitle: organizationName,
      onTap: onTap,
    );
  }
}
