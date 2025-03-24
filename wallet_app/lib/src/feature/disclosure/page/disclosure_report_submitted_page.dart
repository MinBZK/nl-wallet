import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class DisclosureReportSubmittedPage extends StatelessWidget {
  final VoidCallback onClosePressed;

  const DisclosureReportSubmittedPage({
    required this.onClosePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.disclosureReportSubmittedPageTitle,
      description: context.l10n.disclosureReportSubmittedPageSubtitle,
      primaryButtonCta: context.l10n.disclosureReportSubmittedPageCloseCta,
      onPrimaryPressed: onClosePressed,
      illustration: const PageIllustration(asset: WalletAssets.svg_sharing_success),
    );
  }
}
