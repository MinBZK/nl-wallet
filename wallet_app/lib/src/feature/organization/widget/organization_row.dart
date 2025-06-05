import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/menu_item.dart';
import '../../common/widget/organization/organization_logo.dart';

class OrganizationRow extends StatelessWidget {
  final Organization organization;
  final VoidCallback? onPressed;

  const OrganizationRow({
    required this.organization,
    this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return MenuItem(
      leftIcon: OrganizationLogo(image: organization.logo, size: kMenuItemNormalIconSize),
      label: Text.rich(context.l10n.organizationButtonLabel.toTextSpan(context)),
      subtitle: Text.rich(organization.displayName.l10nSpan(context)),
      onPressed: onPressed,
    );
  }
}
