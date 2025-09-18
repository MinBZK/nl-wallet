import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'button_content.dart';
import 'confirm/confirm_buttons.dart';

class SecondaryButton extends StatelessWidget implements FitsWidthWidget {
  final VoidCallback? onPressed;
  final Text text;
  final Widget? icon;
  final IconPosition iconPosition;
  final MainAxisAlignment mainAxisAlignment;

  const SecondaryButton({
    this.onPressed,
    required this.text,
    this.icon = const Icon(Icons.arrow_forward_outlined),
    this.iconPosition = IconPosition.start,
    this.mainAxisAlignment = MainAxisAlignment.center,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return OutlinedButton(
      onPressed: onPressed,
      child: _buildContent(),
    );
  }

  ButtonContent _buildContent() => ButtonContent(
    text: text,
    icon: icon,
    iconPosition: iconPosition,
    mainAxisAlignment: mainAxisAlignment,
  );

  @override
  bool fitsWidth(BuildContext context, double availableWidth) {
    final leftOverWidth = availableWidth - context.theme.buttonTheme.padding.horizontal;
    final contentWidth = _buildContent().contentWidth(
      context,
      context.theme.outlinedButtonTheme.style!.textStyle!.resolve({})!,
    );
    return contentWidth <= leftOverWidth;
  }
}
