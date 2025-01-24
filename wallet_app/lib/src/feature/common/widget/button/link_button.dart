import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'button_content.dart';

const _kButtonHeight = 48.0;

/// A button with a trailing arrow that somewhat resembles a hyperlink with its behaviour.
/// i.e. it has no ripple effect and the text color changes when it's in a pressed state.
class LinkButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final Text text;
  final Widget? icon;
  final IconPosition iconPosition;
  final MainAxisAlignment mainAxisAlignment;

  const LinkButton({
    this.onPressed,
    required this.text,
    this.icon = const Icon(Icons.arrow_forward_outlined),
    this.iconPosition = IconPosition.end,
    this.mainAxisAlignment = MainAxisAlignment.start,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TextButton(
      onPressed: onPressed,
      style: _resolveButtonStyle(context),
      child: ButtonContent(
        text: text,
        iconPosition: IconPosition.end,
        icon: icon,
        mainAxisAlignment: MainAxisAlignment.start,
      ),
    );
  }

  ButtonStyle _resolveButtonStyle(BuildContext context) => context.theme.textButtonTheme.style!.copyWith(
        minimumSize: const WidgetStatePropertyAll(
          Size(0, _kButtonHeight),
        ),
        padding: const WidgetStatePropertyAll(
          EdgeInsets.symmetric(horizontal: 0, vertical: 8),
        ),
        shape: const WidgetStatePropertyAll(
          RoundedRectangleBorder(
            borderRadius: BorderRadius.zero,
          ),
        ),
      );
}
