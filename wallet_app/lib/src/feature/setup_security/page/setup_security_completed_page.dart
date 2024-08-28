import 'dart:io';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class SetupSecurityCompletedPage extends StatelessWidget {
  final VoidCallback onSetupWalletPressed;
  final bool biometricsEnabled;

  const SetupSecurityCompletedPage({
    required this.onSetupWalletPressed,
    this.biometricsEnabled = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TerminalPage(
      title: context.l10n.setupSecurityCompletedPageTitle,
      primaryButtonCta: context.l10n.setupSecurityCompletedPageCreateWalletCta,
      description: _resolveDescription(context),
      onPrimaryPressed: onSetupWalletPressed,
      illustration: const PageIllustration(asset: WalletAssets.svg_pin_set),
    );
  }

  String _resolveDescription(BuildContext context) {
    if (!biometricsEnabled) return context.l10n.setupSecurityCompletedPageDescription;
    if (Platform.isIOS) return context.l10n.setupSecurityCompletedPageWithBiometricsDescriptioniOSVariant;
    return context.l10n.setupSecurityCompletedPageWithBiometricsDescription;
  }
}
