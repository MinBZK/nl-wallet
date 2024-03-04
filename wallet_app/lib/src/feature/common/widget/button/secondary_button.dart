import 'package:flutter/material.dart';

import 'text_icon_button.dart';

const _kButtonMinHeight = 48.0;

class SecondaryButton extends StatelessWidget {
  final VoidCallback onPressed;
  final String text;
  final IconData icon;

  const SecondaryButton({
    required this.onPressed,
    required this.text,
    this.icon = Icons.arrow_forward,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ConstrainedBox(
      constraints: const BoxConstraints(
        minHeight: _kButtonMinHeight,
        minWidth: double.infinity,
      ),
      child: TextIconButton(
        onPressed: onPressed,
        icon: icon,
        iconPosition: IconPosition.start,
        centerChild: false,
        child: Text(text),
      ),
    );
  }
}
