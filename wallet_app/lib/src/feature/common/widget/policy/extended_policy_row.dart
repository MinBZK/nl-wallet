import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

const double _kIconSize = 24;

class ExtendedPolicyRow extends StatelessWidget {
  final IconData icon;
  final String title;

  const ExtendedPolicyRow({
    required this.icon,
    required this.title,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            icon,
            color: context.colorScheme.onBackground,
            size: _kIconSize,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                ConstrainedBox(
                  constraints: const BoxConstraints(minHeight: _kIconSize),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      title,
                      style: context.textTheme.titleMedium,
                    ),
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  '...',
                  style: context.textTheme.bodyLarge,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
