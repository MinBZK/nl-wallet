import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'button_content.dart';

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
        minimumSize: MaterialStateProperty.all(
          const Size(0, 48),
        ),
        shape: MaterialStateProperty.all(
          const LinearBorder(),
        ),
        overlayColor: MaterialStateProperty.all(Colors.transparent),
        splashFactory: NoSplash.splashFactory,
        foregroundColor: MaterialStateProperty.resolveWith(
          _getForegroundColor(context),
        ),
        animationDuration: Duration.zero,
        padding: MaterialStateProperty.all(
          const EdgeInsets.symmetric(horizontal: 0, vertical: 8),
        ),
      );

  Color Function(Set<MaterialState> states) _getForegroundColor(BuildContext context) {
    return (Set<MaterialState> states) {
      const Set<MaterialState> interactiveStates = <MaterialState>{
        MaterialState.pressed,
        MaterialState.hovered,
        MaterialState.focused,
      };
      if (states.any(interactiveStates.contains)) {
        return context.theme.primaryColorLight;
      }
      return context.theme.textButtonTheme.style?.foregroundColor?.resolve(states) ?? context.colorScheme.primary;
    };
  }
}
