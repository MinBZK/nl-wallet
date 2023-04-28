import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/info_row.dart';

class OrganizationRow extends StatelessWidget {
  final String organizationName;
  final VoidCallback? onTap;

  const OrganizationRow({
    required this.organizationName,
    this.onTap,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return InfoRow(
      icon: Icons.apartment,
      title: AppLocalizations.of(context).organizationButtonLabel,
      subtitle: organizationName,
      onTap: onTap,
    );
  }
}
