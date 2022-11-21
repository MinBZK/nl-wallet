import 'package:flutter/material.dart';

import '../../verification/widget/status_icon.dart';
import 'text_icon_button.dart';

/// Base widget for the terminal (ending) page of the issuance/verification flow.
class FlowTerminalPage extends StatelessWidget {
  final IconData icon;
  final Color? iconColor;
  final String title;
  final String description;
  final String? secondaryButtonCta;
  final VoidCallback? onSecondaryButtonPressed;
  final String? tertiaryButtonCta;
  final VoidCallback? onTertiaryButtonPressed;
  final String closeButtonCta;
  final VoidCallback onClosePressed;

  const FlowTerminalPage({
    required this.icon,
    this.iconColor,
    required this.title,
    required this.description,
    this.secondaryButtonCta,
    this.onSecondaryButtonPressed,
    this.tertiaryButtonCta,
    this.onTertiaryButtonPressed,
    required this.closeButtonCta,
    required this.onClosePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const SizedBox(height: 24),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: StatusIcon(
              icon: icon,
              color: iconColor,
            ),
          ),
          const SizedBox(height: 32),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              title,
              style: Theme.of(context).textTheme.headline2,
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              description,
              style: Theme.of(context).textTheme.bodyText1,
              textAlign: TextAlign.center,
            ),
          ),
          if (tertiaryButtonCta != null)
            TextIconButton(
              onPressed: onTertiaryButtonPressed,
              child: Text(tertiaryButtonCta!),
            ),
          const Spacer(),
          if (secondaryButtonCta != null)
            TextIconButton(
              onPressed: onSecondaryButtonPressed,
              child: Text(secondaryButtonCta!),
            ),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: ElevatedButton(
              onPressed: onClosePressed,
              child: Text(closeButtonCta),
            ),
          ),
        ],
      ),
    );
  }
}
