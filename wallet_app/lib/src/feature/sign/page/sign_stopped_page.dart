import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
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
      primaryButton: PrimaryButton(
        text: Text(context.l10n.signStoppedPageCloseCta),
        onPressed: onClosePressed,
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.signStoppedPageFeedbackCta),
        icon: const Icon(Icons.arrow_forward_outlined),
        onPressed: onGiveFeedbackPressed,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }
}
