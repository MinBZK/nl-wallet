import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/page_illustration.dart';

class SignSuccessPage extends StatelessWidget {
  final LocalizedText organizationName;
  final VoidCallback? onHistoryPressed;
  final VoidCallback onClosePressed;

  const SignSuccessPage({
    required this.organizationName,
    required this.onClosePressed,
    this.onHistoryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.signSuccessPageTitle,
      description: context.l10n.signSuccessPageDescription(organizationName.l10nValue(context)),
      illustration: const PageIllustration(asset: WalletAssets.svg_signed),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.signSuccessPageCloseCta),
        onPressed: onClosePressed,
        key: const Key('primaryButtonCta'),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.signSuccessPageHistoryCta),
        icon: const Icon(Icons.arrow_forward_outlined),
        onPressed: onHistoryPressed,
        key: const Key('secondaryButtonCta'),
      ),
    );
  }
}
