import 'package:flutter/material.dart';

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
            color: Theme.of(context).colorScheme.primary,
          ),
          text: const Text('colorScheme.primary'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.primaryContainer,
          ),
          text: const Text('colorScheme.primaryContainer'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.onPrimary,
          ),
          text: const Text('colorScheme.onPrimary'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.secondary,
          ),
          text: const Text('colorScheme.secondary'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.secondaryContainer,
          ),
          text: const Text('colorScheme.secondaryContainer'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.background,
          ),
          text: const Text('colorScheme.background'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.onBackground,
          ),
          text: const Text('colorScheme.onBackground'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.error,
          ),
          text: const Text('colorScheme.error'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.onSurface,
          ),
          text: const Text('colorScheme.onSurface'),
        ),
        IconRow(
          icon: Icon(
            Icons.circle,
            color: Theme.of(context).colorScheme.outlineVariant,
          ),
          text: const Text('colorScheme.outlineVariant'),
        ),
      ],
    );
  }
}
