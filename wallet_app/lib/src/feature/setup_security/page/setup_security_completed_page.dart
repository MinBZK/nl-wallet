import 'package:flutter/material.dart';

import '../../../domain/usecase/biometrics/biometrics.dart';
import '../../../util/extension/biometrics_extension.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/page_illustration.dart';
import '../../common/page/terminal_page.dart';

class SetupSecurityCompletedPage extends StatelessWidget {
  final VoidCallback onSetupWalletPressed;
  final Biometrics enabledBiometrics;

  const SetupSecurityCompletedPage({
    required this.onSetupWalletPressed,
    this.enabledBiometrics = Biometrics.none,
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
    if (enabledBiometrics == Biometrics.none) return context.l10n.setupSecurityCompletedPageDescription;
    final biometrics = enabledBiometrics.prettyPrint(context);
    return context.l10n.setupSecurityCompletedPageWithBiometricsDescription(biometrics);
  }
}
