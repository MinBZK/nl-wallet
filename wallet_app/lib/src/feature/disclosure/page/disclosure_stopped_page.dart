import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/page/terminal_page.dart';

class DisclosureStoppedPage extends StatelessWidget {
  final Organization organization;
  final VoidCallback onClosePressed;

  const DisclosureStoppedPage({
    required this.onClosePressed,
    required this.organization,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.disclosureStoppedPageTitle,
      description: context.l10n.disclosureStoppedPageDescription(organization.displayName.l10nValue(context)),
      primaryButtonCta: context.l10n.disclosureStoppedPageCloseCta,
      onPrimaryPressed: onClosePressed,
    );
  }
}
