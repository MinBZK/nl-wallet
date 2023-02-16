import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class KeyboardBackspaceKey extends StatelessWidget {
  final VoidCallback? onBackspacePressed;

  const KeyboardBackspaceKey({this.onBackspacePressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Semantics(
        button: true,
        label: AppLocalizations.of(context).pinKeyboardWCAGBackspaceLabel,
        child: InkWell(
          onTap: onBackspacePressed == null ? null : () => onBackspacePressed!(),
          child: const Icon(Icons.backspace),
        ),
      ),
    );
  }
}
