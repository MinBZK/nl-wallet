import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class PolicyEntryRow extends StatelessWidget {
  /// The title of the section, usually a [TitleText] widget with textTheme set to titleMedium
  final Widget title;

  /// The description of the section, usually a [BodyText] widget
  final Widget description;

  /// An optional descriptive icon, usually an [Icon] widget
  final Widget? icon;

  const PolicyEntryRow({
    required this.title,
    required this.description,
    this.icon,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          IconTheme(
            data: context.theme.iconTheme.copyWith(
              size: context.textScaler.scale(24) /* scale icon with fontSize, to keep it aligned with the title */,
              color: context.colorScheme.onSurfaceVariant,
            ),
            child: icon ?? const SizedBox.shrink(),
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
          ),
        ],
      ),
    );
  }
}
