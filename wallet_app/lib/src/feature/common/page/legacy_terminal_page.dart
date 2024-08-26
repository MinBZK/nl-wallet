import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/button/button_content.dart';
import '../widget/button/link_button.dart';
import '../widget/button/primary_button.dart';
import '../widget/button/tertiary_button.dart';
import '../widget/status_icon.dart';
import '../widget/wallet_scrollbar.dart';

/// Base widget for the terminal (ending) page of the issuance/disclosure flow.
class LegacyTerminalPage extends StatelessWidget {
  final IconData icon;
  final Color? iconColor;
  final String title;
  final String description;
  final String? secondaryButtonCta;
  final VoidCallback? onSecondaryButtonPressed;
  final String? tertiaryButtonCta;
  final VoidCallback? onTertiaryButtonPressed;
  final String primaryButtonCta;
  final VoidCallback onPrimaryPressed;
  final Widget? content;

  const LegacyTerminalPage({
    required this.icon,
    this.iconColor,
    required this.title,
    required this.description,
    this.secondaryButtonCta,
    this.onSecondaryButtonPressed,
    this.tertiaryButtonCta,
    this.onTertiaryButtonPressed,
    this.content,
    required this.primaryButtonCta,
    required this.onPrimaryPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildScrollableSection(context),
          const SizedBox(height: 16),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Expanded(
      child: WalletScrollbar(
        child: ListView(
          padding: const EdgeInsets.symmetric(vertical: 24),
          children: [
            const SizedBox(height: 24),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Center(
                child: StatusIcon(
                  icon: icon,
                  color: iconColor,
                ),
              ),
            ),
            const SizedBox(height: 16),
            Padding(
              padding: const EdgeInsets.all(16),

              /// The [Column] is added to improve semantics; reading title and description together
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text.rich(
                    title.toTextSpan(context),
                    style: context.textTheme.displayMedium,
                    textAlign: TextAlign.start,
                  ),
                  const SizedBox(height: 8),
                  Text.rich(
                    description.toTextSpan(context),
                    style: context.textTheme.bodyLarge,
                    textAlign: TextAlign.start,
                  ),
                ],
              ),
            ),
            if (tertiaryButtonCta != null)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: LinkButton(
                  onPressed: onTertiaryButtonPressed,
                  text: Text.rich(tertiaryButtonCta!.toTextSpan(context)),
                ),
              ),
            if (content != null) content!,
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    Widget? secondaryButton;
    if (secondaryButtonCta != null) {
      secondaryButton = TertiaryButton(
        onPressed: onSecondaryButtonPressed,
        text: Text(secondaryButtonCta!),
        iconPosition: IconPosition.end,
      );
    }
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Divider(height: 1),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
          child: Column(
            children: [
              if (secondaryButton != null) secondaryButton,
              if (secondaryButton != null) const SizedBox(height: 12),
              PrimaryButton(
                key: const Key('primaryButtonCta'),
                onPressed: onPrimaryPressed,
                text: Text(primaryButtonCta),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
