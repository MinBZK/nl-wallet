import 'package:flutter/material.dart';

import 'link_button.dart';

/// A button that encapsulates the normal [LinkButton] by wrapping it with
/// a top and bottom divider, some vertical padding and makes the whole row
/// clickable with visual feedback.
class LinkTileButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final Widget child;

  const LinkTileButton({
    required this.child,
    this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onPressed,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Divider(height: 1),
          const SizedBox(height: 12),
          LinkButton(
            customPadding: const EdgeInsets.only(left: 16),
            child: child,
          ),
          const SizedBox(height: 12),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
