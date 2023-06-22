import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class InfoRow extends StatelessWidget {
  final String? title;
  final String? subtitle;
  final IconData? icon;
  final Widget? leading;
  final VoidCallback? onTap;

  const InfoRow({
    this.title,
    this.subtitle,
    this.leading,
    this.icon,
    this.onTap,
    Key? key,
  })  : assert(leading == null || icon == null, 'You cannot provide a leading widget and an icon'),
        assert(leading != null || icon != null, 'Provide a leading widget or icon'),
        super(key: key);

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            leading ??
                Icon(
                  icon,
                  color: context.colorScheme.primary,
                ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (title != null)
                    Text(
                      title!,
                      style: context.textTheme.titleMedium,
                    ),
                  if (subtitle != null)
                    Text(
                      subtitle!,
                      style: context.textTheme.bodyMedium,
                    ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            if (onTap != null)
              Icon(
                Icons.chevron_right,
                color: context.theme.primaryColorDark,
              ),
          ],
        ),
      ),
    );
  }
}
