import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
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
    return Semantics(
      label: context.l10n.setupSecurityScreenWCAGEnteredDigitsAnnouncement(enteredDigits, digits),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(
          digits,
          (index) => PinDot(
            checked: index < enteredDigits,
            key: ValueKey(index),
            color: context.colorScheme.onBackground,
          ),
        ),
      ),
    );
  }
}
