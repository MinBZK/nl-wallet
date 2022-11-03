import 'package:flutter/material.dart';

/// A button with a trailing arrow that somewhat resembles a hyperlink with its behaviour.
/// i.e. it has no ripple effect and the text color changes when it's in a pressed state.
class LinkButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;

  const LinkButton({
    required this.child,
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return TextButton(
      onPressed: onPressed,
      style: ButtonStyle(
        overlayColor: MaterialStateProperty.all(Colors.transparent),
        splashFactory: NoSplash.splashFactory,
        foregroundColor: MaterialStateProperty.resolveWith(_getForegroundColor(context)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          child,
          const SizedBox(width: 8),
          const Icon(Icons.arrow_forward, size: 12),
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
        return Theme.of(context).primaryColorLight;
      }
      return Theme.of(context).textButtonTheme.style?.foregroundColor?.resolve(states) ??
          Theme.of(context).colorScheme.primary;
    };
  }
}
