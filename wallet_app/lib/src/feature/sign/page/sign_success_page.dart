import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
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
      primaryButtonCta: context.l10n.signSuccessPageCloseCta,
      illustration: const PageIllustration(asset: WalletAssets.svg_signed),
      onPrimaryPressed: onClosePressed,
      secondaryButtonCta: context.l10n.signSuccessPageHistoryCta,
      onSecondaryButtonPressed: onHistoryPressed,
    );
  }
}
