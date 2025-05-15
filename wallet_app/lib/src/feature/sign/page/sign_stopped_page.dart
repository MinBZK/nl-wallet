import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/page_illustration.dart';

class SignStoppedPage extends StatelessWidget {
  final VoidCallback? onGiveFeedbackPressed;
  final VoidCallback onClosePressed;

  const SignStoppedPage({
    required this.onClosePressed,
    this.onGiveFeedbackPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      illustration: const PageIllustration(asset: WalletAssets.svg_stopped),
      title: context.l10n.signStoppedPageTitle,
      description: context.l10n.signStoppedPageDescription,
      primaryButtonCta: context.l10n.signStoppedPageCloseCta,
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.signStoppedPageFeedbackCta,
      onSecondaryButtonPressed: onGiveFeedbackPressed,
    );
  }
}
