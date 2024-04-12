import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

/// A Button that spans the full width of the screen and wraps the [child] with optional bottom and top dividers.
class LinkTileButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final Widget child;
  final bool showDividers;

  const LinkTileButton({
    required this.child,
    this.onPressed,
    this.showDividers = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onPressed,
      child: IntrinsicHeight(
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: 72),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              showDividers ? const Divider(height: 1) : const SizedBox.shrink(),
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                  child: Row(
                    children: [
                      DefaultTextStyle(
                        style: context.textTheme.labelLarge!.copyWith(color: context.colorScheme.primary),
                        child: child,
                      ),
                      const SizedBox(width: 12),
                      Icon(
                        Icons.arrow_forward_rounded,
                        size: 16,
                        color: context.colorScheme.primary,
                      ),
                    ],
                  ),
                ),
              ),
              showDividers ? const Divider(height: 1) : const SizedBox.shrink(),
            ],
          ),
        ),
      ),
    );
  }
}
