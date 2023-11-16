import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/link_button.dart';
import '../widget/button/text_icon_button.dart';
import '../widget/status_icon.dart';

/// Base widget for the terminal (ending) page of the issuance/disclosure flow.
class FlowTerminalPage extends StatelessWidget {
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

  const FlowTerminalPage({
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
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildScrollableSection(context),
        const SizedBox(height: 16),
        _buildBottomSection(context),
      ],
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Expanded(
      child: Scrollbar(
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
                  Text(
                    title,
                    style: context.textTheme.displayMedium,
                    textAlign: TextAlign.start,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    description,
                    style: context.textTheme.bodyLarge,
                    textAlign: TextAlign.start,
                  )
                ],
              ),
            ),
            if (tertiaryButtonCta != null)
              LinkButton(
                customPadding: const EdgeInsets.symmetric(horizontal: 16),
                onPressed: onTertiaryButtonPressed,
                child: Text(tertiaryButtonCta!),
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
      secondaryButton = TextIconButton(
        onPressed: onSecondaryButtonPressed,
        child: Text(secondaryButtonCta!),
      );
    }
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (secondaryButton != null) secondaryButton,
        if (secondaryButton != null) const SizedBox(height: 16),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: ElevatedButton(
            key: const Key('primaryButtonCta'),
            onPressed: onPrimaryPressed,
            child: Text(primaryButtonCta),
          ),
        ),
        const SizedBox(height: 16),
      ],
    );
  }
}
