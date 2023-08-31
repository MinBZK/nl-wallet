import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/icon_row.dart';

class ColorStylesTab extends StatelessWidget {
  const ColorStylesTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
      children: [
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.primary,
          ),
          text: const Text('colorScheme.primary'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.primaryContainer,
          ),
          text: const Text('colorScheme.primaryContainer'),
        ),
        Container(
          color: context.colorScheme.primary,
          child: IconRow(
            icon: Icon(
              Icons.circle,
              color: context.colorScheme.onPrimary,
            ),
            text: Text(
              'colorScheme.onPrimary',
              style: context.textTheme.bodyLarge?.copyWith(
                color: context.colorScheme.onPrimary,
              ),
            ),
          ),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.secondary,
          ),
          text: const Text('colorScheme.secondary'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.secondaryContainer,
          ),
          text: const Text('colorScheme.secondaryContainer'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.background,
          ),
          text: const Text('colorScheme.background'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.onBackground,
          ),
          text: const Text('colorScheme.onBackground'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.error,
          ),
          text: const Text('colorScheme.error'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.onSurface,
          ),
          text: const Text('colorScheme.onSurface'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: context.colorScheme.outlineVariant,
          ),
          text: const Text('colorScheme.outlineVariant'),
        ),
      ],
    );
  }
}
