import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/store/active_locale_provider.dart';
import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';

const _kMinHeight = 72.0;
const _kIconSize = 24.0;

class MenuRow extends StatelessWidget {
  final IconData? icon;
  final String label;
  final String? subtitle;
  final VoidCallback? onTap;

  const MenuRow({
    super.key,
    this.icon,
    required this.label,
    this.subtitle,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: _kMinHeight),
        child: TextButton.icon(
          onPressed: onTap,
          icon: _buildIcon(context),
          iconAlignment: IconAlignment.end,
          style: context.theme.iconButtonTheme.style?.copyWith(
            foregroundColor: WidgetStateProperty.resolveWith(
              // Only override the color when the button is not pressed or focused
              (states) => states.isPressedOrFocused ? null : context.colorScheme.onSurface,
            ),
            shape: WidgetStateProperty.all(
              const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
            ),
          ),
          label: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              _buildLeading(context),
              _buildContent(context),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildLeading(BuildContext context) {
    if (icon == null) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.only(right: 16),
      alignment: Alignment.center,
      child: Icon(
        icon,
        color: context.colorScheme.onSurfaceVariant,
        size: _kIconSize,
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    if (subtitle == null) {
      return Expanded(
        child: Text.rich(
          TextSpan(
            text: label,
            locale: context.read<ActiveLocaleProvider>().activeLocale,
          ),
        ),
      );
    }

    return Expanded(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            label,
            style: context.textTheme.titleMedium,
          ),
          Text(
            subtitle ?? '',
            style: context.textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }

  Widget? _buildIcon(BuildContext context) {
    if (onTap == null) return null;
    return const Icon(Icons.chevron_right);
  }
}
