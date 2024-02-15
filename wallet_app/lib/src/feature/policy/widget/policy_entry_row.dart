import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class PolicyEntryRow extends StatelessWidget {
  final IconData? icon;
  final Widget title;
  final Widget description;

  const PolicyEntryRow({
    this.icon,
    required this.title,
    required this.description,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          icon == null
              ? const SizedBox.shrink()
              : Icon(
                  icon,
                  size: 24,
                  color: context.colorScheme.onSurface,
                ),
          SizedBox(width: icon == null ? 0 : 16),
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ConstrainedBox(
                  constraints: const BoxConstraints(minHeight: 24),
                  child: DefaultTextStyle(
                    style: context.textTheme.titleMedium!,
                    child: title,
                  ),
                ),
                const SizedBox(height: 8),
                DefaultTextStyle(
                  style: context.textTheme.bodyLarge!,
                  child: description,
                ),
              ],
            ),
          )
        ],
      ),
    );
  }
}
