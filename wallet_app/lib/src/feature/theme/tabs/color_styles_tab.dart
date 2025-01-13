import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/icon_row.dart';

class ColorStylesTab extends StatelessWidget {
  const ColorStylesTab({super.key});

  @override
  Widget build(BuildContext context) {
    final colorScheme = context.colorScheme;
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      children: [
        _ColorRow(
          name: 'colorScheme.primary',
          color: colorScheme.primary,
        ),
        _ColorRow(
          name: 'colorScheme.onPrimary',
          color: colorScheme.onPrimary,
          bgColor: colorScheme.primary,
          textColor: colorScheme.onPrimary,
        ),
        _ColorRow(
          name: 'colorScheme.primaryContainer',
          color: colorScheme.primaryContainer,
        ),
        _ColorRow(
          name: 'colorScheme.onPrimaryContainer',
          color: colorScheme.onPrimaryContainer,
          bgColor: colorScheme.primaryContainer,
          textColor: colorScheme.onPrimaryContainer,
        ),
        _ColorRow(
          name: 'colorScheme.secondary',
          color: colorScheme.secondary,
        ),
        _ColorRow(
          name: 'colorScheme.onSecondary',
          color: colorScheme.onSecondary,
          bgColor: colorScheme.secondary,
          textColor: colorScheme.onSecondary,
        ),
        _ColorRow(
          name: 'colorScheme.secondaryContainer',
          color: colorScheme.secondaryContainer,
        ),
        _ColorRow(
          name: 'colorScheme.onSecondaryContainer',
          color: colorScheme.onSecondaryContainer,
          bgColor: colorScheme.secondaryContainer,
          textColor: colorScheme.onSecondaryContainer,
        ),
        _ColorRow(
          name: 'colorScheme.tertiary',
          color: colorScheme.tertiary,
        ),
        _ColorRow(
          name: 'colorScheme.onTertiary',
          color: colorScheme.onTertiary,
          bgColor: colorScheme.tertiary,
          textColor: colorScheme.onTertiary,
        ),
        _ColorRow(
          name: 'colorScheme.tertiaryContainer',
          color: colorScheme.tertiaryContainer,
        ),
        _ColorRow(
          name: 'colorScheme.onTertiaryContainer',
          color: colorScheme.onTertiaryContainer,
          bgColor: colorScheme.tertiaryContainer,
          textColor: colorScheme.onTertiaryContainer,
        ),
        _ColorRow(
          name: 'colorScheme.error',
          color: colorScheme.error,
        ),
        _ColorRow(
          name: 'colorScheme.onError',
          color: colorScheme.onError,
          bgColor: colorScheme.error,
          textColor: colorScheme.onError,
        ),
        _ColorRow(
          name: 'colorScheme.errorContainer',
          color: colorScheme.errorContainer,
        ),
        _ColorRow(
          name: 'colorScheme.onErrorContainer',
          color: colorScheme.onErrorContainer,
          bgColor: colorScheme.errorContainer,
          textColor: colorScheme.onErrorContainer,
        ),
        _ColorRow(
          name: 'colorScheme.surface',
          color: colorScheme.surface,
        ),
        _ColorRow(
          name: 'colorScheme.onSurface',
          color: colorScheme.onSurface,
          bgColor: colorScheme.surface,
          textColor: colorScheme.onSurface,
        ),
        _ColorRow(
          name: 'colorScheme.surfaceTint',
          color: colorScheme.surfaceTint,
        ),
        _ColorRow(
          name: 'colorScheme.surfaceContainerHighest',
          color: colorScheme.surfaceContainerHighest,
        ),
        _ColorRow(
          name: 'colorScheme.onSurfaceVariant',
          color: colorScheme.onSurfaceVariant,
          bgColor: colorScheme.surfaceContainerHighest,
          textColor: colorScheme.onSurfaceVariant,
        ),
        _ColorRow(
          name: 'colorScheme.outline',
          color: colorScheme.outline,
        ),
        _ColorRow(
          name: 'colorScheme.outlineVariant',
          color: colorScheme.outlineVariant,
        ),
        _ColorRow(
          name: 'colorScheme.scrim',
          color: colorScheme.scrim,
        ),
        _ColorRow(
          name: 'colorScheme.shadow',
          color: colorScheme.shadow,
        ),
      ],
    );
  }
}

/// Simple widget to demonstrate a color, only used in the design system color tab.
class _ColorRow extends StatelessWidget {
  final Color color;
  final Color? bgColor;
  final Color? textColor;
  final String name;

  const _ColorRow({
    required this.name,
    required this.color,
    this.bgColor,
    this.textColor,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      color: bgColor,
      child: IconRow(
        icon: Container(
          height: 36,
          width: 36,
          decoration: BoxDecoration(
            color: color,
            shape: BoxShape.circle,
            border: Border.all(color: Colors.black, width: 2),
          ),
        ),
        text: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              name,
              style: context.textTheme.bodyLarge?.copyWith(color: textColor),
            ),
            Text(
              color.toHex(),
              style: context.textTheme.bodySmall?.copyWith(color: textColor),
            ),
          ],
        ),
      ),
    );
  }
}

extension _HexColor on Color {
  String toHex() {
    final alpha = (a * 255).toInt().toRadixString(16).padLeft(2, '0');
    final red = (r * 255).toInt().toRadixString(16).padLeft(2, '0');
    final green = (g * 255).toInt().toRadixString(16).padLeft(2, '0');
    final blue = (b * 255).toInt().toRadixString(16).padLeft(2, '0');

    return '#$alpha$red$green$blue'.toUpperCase();
  }
}
