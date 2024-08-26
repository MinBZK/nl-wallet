import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

class PolicyRow extends StatelessWidget {
  final IconData icon;
  final String title;

  const PolicyRow({
    required this.icon,
    required this.title,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Icon(
            icon,
            color: context.colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Text.rich(
              title.toTextSpan(context),
              style: context.textTheme.bodyLarge,
            ),
          ),
        ],
      ),
    );
  }
}
