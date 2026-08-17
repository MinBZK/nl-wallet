import 'package:flutter/material.dart';

import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/formatter/country_code_formatter.dart';
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
    final countryLabel = CountryCodeFormatter.format(organization.countryCode);
    return MenuItem(
      leftIcon: organization.logo == null
          ? null
          : OrganizationLogo(image: organization.logo!, size: kMenuItemNormalIconSize),
      label: Text.rich(
        context.l10n.requestDetailScreenAboutOrganizationCta(organization.displayName).toTextSpan(context),
      ),
      subtitle: countryLabel == null ? null : Text.rich(countryLabel.toTextSpan(context)),
      onPressed: onPressed,
    );
  }
}
