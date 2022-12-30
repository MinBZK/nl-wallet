import 'package:flutter/material.dart';

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
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(
        digits,
        (index) => PinDot(
          checked: index < enteredDigits,
          key: ValueKey(index),
          color: Theme.of(context).colorScheme.onBackground,
        ),
      ),
    );
  }
}
