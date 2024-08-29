import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/store/active_locale_provider.dart';
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
        child: InkWell(
          onTap: onTap,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              _buildLeading(context),
              _buildContent(context),
              _buildTrailing(context),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildLeading(BuildContext context) {
    const edgeInsets = EdgeInsets.symmetric(horizontal: 16, vertical: 24);
    if (icon == null) return const SizedBox(width: 16);
    return Container(
      padding: edgeInsets,
      alignment: Alignment.center,
      child: Icon(
        icon,
        color: context.colorScheme.onSurfaceVariant,
        size: _kIconSize,
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    const verticalPadding = EdgeInsets.symmetric(vertical: 24);

    if (subtitle == null) {
      return Expanded(
        child: Padding(
          padding: verticalPadding,
          child: Text.rich(
            TextSpan(text: label, locale: context.read<ActiveLocaleProvider>().activeLocale),
            style: context.textTheme.titleMedium,
          ),
        ),
      );
    }

    return Expanded(
      child: Padding(
        padding: verticalPadding,
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
      ),
    );
  }

  Widget _buildTrailing(BuildContext context) {
    if (onTap == null) return const SizedBox(width: 16);
    const edgeInsets = EdgeInsets.symmetric(horizontal: 16, vertical: 24);
    return Container(
      padding: edgeInsets,
      alignment: Alignment.center,
      child: Icon(
        Icons.chevron_right,
        size: _kIconSize,
        color: context.colorScheme.primary,
      ),
    );
  }
}
