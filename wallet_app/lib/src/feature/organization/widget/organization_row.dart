import 'package:flutter/material.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/organization/organization_logo.dart';

class OrganizationRow extends StatelessWidget {
  final String subtitle;
  final VoidCallback? onTap;
  final AppImageData? image;

  const OrganizationRow({
    required this.subtitle,
    this.onTap,
    this.image,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return InfoRow(
      icon: image != null ? null : Icons.apartment_outlined,
      leading: image != null ? OrganizationLogo(image: image!, size: 24) : null,
      title: Text(context.l10n.organizationButtonLabel),
      subtitle: Text(subtitle),
      onTap: onTap,
    );
  }
}
