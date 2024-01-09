import 'package:flutter/material.dart';

/// Similar to the normal [BackButton] widget, but always uses the [Icons.arrow_back] icon.
class WalletBackButton extends StatelessWidget {
  const WalletBackButton({super.key});

  @override
  Widget build(BuildContext context) {
    return IconButton(
      onPressed: () => Navigator.pop(context),
      icon: const Icon(Icons.arrow_back),
      tooltip: MaterialLocalizations.of(context).backButtonTooltip,
    );
  }
}
