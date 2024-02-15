import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../error/error_page.dart';

class DisclosureGenericErrorPage extends StatelessWidget {
  final VoidCallback onStopPressed;

  const DisclosureGenericErrorPage({
    required this.onStopPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ErrorPage.generic(
      context,
      headline: context.l10n.disclosureGenericErrorPageTitle,
      description: context.l10n.disclosureGenericErrorPageDescription,
      primaryActionText: context.l10n.disclosureGenericErrorPageCloseCta,
      primaryActionIcon: Icons.not_interested_rounded,
      onPrimaryActionPressed: onStopPressed,
    );
  }
}
