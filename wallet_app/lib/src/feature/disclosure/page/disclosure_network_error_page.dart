import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../error/error_page.dart';

class DisclosureNetworkErrorPage extends StatelessWidget {
  final VoidCallback onStopPressed;
  final bool hasInternet;

  const DisclosureNetworkErrorPage({
    required this.onStopPressed,
    required this.hasInternet,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (hasInternet) {
      return ErrorPage.network(
        context,
        primaryActionText: context.l10n.disclosureGenericErrorPageCloseCta,
        primaryActionIcon: Icons.not_interested_rounded,
        onPrimaryActionPressed: onStopPressed,
        showHelpSheetAsSecondaryCta: false,
      );
    } else {
      return ErrorPage.noInternet(
        context,
        onPrimaryActionPressed: onStopPressed,
      );
    }
  }
}
