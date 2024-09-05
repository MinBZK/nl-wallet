import 'package:flutter/material.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
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
      title: Text.rich(context.l10n.organizationButtonLabel.toTextSpan(context)),
      subtitle: Text(subtitle),
      onTap: onTap,
    );
  }
}
