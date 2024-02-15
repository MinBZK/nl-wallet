import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

/// A button with a trailing arrow that somewhat resembles a hyperlink with its behaviour.
/// i.e. it has no ripple effect and the text color changes when it's in a pressed state.
class LinkButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final EdgeInsets? customPadding;

  const LinkButton({
    required this.child,
    this.onPressed,
    this.customPadding,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TextButton(
      onPressed: onPressed,
      style: ButtonStyle(
        padding: customPadding != null ? MaterialStatePropertyAll(customPadding!) : null,
        overlayColor: MaterialStateProperty.all(Colors.transparent),
        splashFactory: NoSplash.splashFactory,
        foregroundColor: MaterialStateProperty.resolveWith(
          _getForegroundColor(context),
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Flexible(child: child),
          const SizedBox(width: 8),
          const Icon(Icons.arrow_forward, size: 16),
        ],
      ),
    );
  }

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
