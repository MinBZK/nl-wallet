import 'package:flutter/material.dart';

import '../../common/widget/text_icon_button.dart';
import '../../verification/widget/status_icon.dart';

class IssuanceTerminalPage extends StatelessWidget {
  final IconData icon;
  final String title;
  final String description;
  final String? secondaryButtonCta;
  final VoidCallback? onSecondaryButtonPressed;
  final String closeButtonCta;
  final VoidCallback onClosePressed;

  const IssuanceTerminalPage({
    required this.icon,
    required this.title,
    required this.description,
    this.secondaryButtonCta,
    this.onSecondaryButtonPressed,
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
              color: Theme.of(context).primaryColorDark,
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
