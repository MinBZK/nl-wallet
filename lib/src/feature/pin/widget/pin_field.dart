import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'pin_dot.dart';

class PinField extends StatelessWidget {
  final int digits;
  final int enteredDigits;

  const PinField({
    required this.digits,
    required this.enteredDigits,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Semantics(
      label: locale.setupSecurityScreenWCAGEnteredDigitsAnnouncement(enteredDigits, digits),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(
          digits,
          (index) => PinDot(
            checked: index < enteredDigits,
            key: ValueKey(index),
            color: Theme.of(context).colorScheme.onBackground,
          ),
        ),
      ),
    );
  }
}
