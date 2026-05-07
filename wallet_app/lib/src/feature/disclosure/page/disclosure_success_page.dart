import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/object_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/page_illustration.dart';
import '../bloc/disclosure_bloc.dart';

class DisclosureSuccessPage extends StatelessWidget {
  /// The visual style and messaging variant of the success page.
  final SuccessStyle style;

  /// The localized name of the organization the data was shared with.
  final LocalizedText organizationDisplayName;

  /// The URL to redirect the user back to, if applicable.
  final String? returnUrl;

  /// Callback triggered when the primary action button is pressed.
  final Function(String?) onPrimaryPressed;

  /// Optional callback to review the request.
  /// When unset the 'Show activity' button is hidden.
  final VoidCallback? onShowActivityPressed;

  const DisclosureSuccessPage({
    this.style = .regular,
    required this.organizationDisplayName,
    this.returnUrl,
    required this.onPrimaryPressed,
    this.onShowActivityPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final orgName = organizationDisplayName.l10nValue(context);
    final description = switch (style) {
      SuccessStyle.regular => context.l10n.disclosureSuccessPageDescription(orgName),
      SuccessStyle.login => context.l10n.disclosureSuccessPageDescriptionForLogin(orgName),
      SuccessStyle.closeProximity => context.l10n.disclosureSuccessPageDescriptionForLogin(orgName),
      SuccessStyle.sameDeviceNoReturnUrl => context.l10n.disclosureSuccessPageDescriptionForSameDeviceNoReturnUrl(
        orgName,
      ),
    };
    return TerminalPage(
      title: context.l10n.disclosureSuccessPageTitle,
      description: description,
      illustration: const PageIllustration(asset: WalletAssets.svg_sharing_success),
      primaryButton: _buildPrimaryButton(context),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.disclosureSuccessPageShowHistoryCta),
        icon: const Icon(Icons.arrow_forward_outlined),
        onPressed: onShowActivityPressed,
        key: const Key('secondaryButtonCta'),
      ).takeIf((_) => onShowActivityPressed != null),
    );
  }

  FitsWidthWidget _buildPrimaryButton(BuildContext context) {
    final bool hasReturnUrl = returnUrl != null;
    final primaryButtonCta = hasReturnUrl
        ? context.l10n.disclosureSuccessPageToWebsiteCta
        : context.l10n.disclosureSuccessPageToDashboardCta;
    switch (style) {
      case SuccessStyle.regular:
      case SuccessStyle.login:
      case SuccessStyle.closeProximity:
        return PrimaryButton(
          text: Text(primaryButtonCta),
          icon: Icon(hasReturnUrl ? Icons.north_east : Icons.arrow_forward),
          onPressed: () => onPrimaryPressed(returnUrl),
          key: const Key('primaryButtonCta'),
        );
      case SuccessStyle.sameDeviceNoReturnUrl:
        return SecondaryButton(
          text: Text(context.l10n.disclosureSuccessPageToDashboardCta),
          icon: const Icon(Icons.arrow_forward),
          onPressed: () => onPrimaryPressed(null),
          key: const Key('primaryButtonCta'),
        );
    }
  }
}
