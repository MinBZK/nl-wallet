import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class KeyboardDigitKey extends StatelessWidget {
  final int digit;
  final Function(int)? onKeyPressed;

  const KeyboardDigitKey({required this.digit, this.onKeyPressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: InkWell(
        onTap: onKeyPressed == null ? null : () => onKeyPressed!(digit),
        child: Center(
          child: Semantics(
            keyboardKey: true,
            onTapHint: AppLocalizations.of(context).pinKeyboardWCAGDigitKeyTapHint,
            child: Text(
              digit.toString(),
              textAlign: TextAlign.center,
            ),
          ),
        ),
      ),
    );
  }
}
