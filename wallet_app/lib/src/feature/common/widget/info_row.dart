import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class InfoRow extends StatelessWidget {
  final Text? title;
  final Text? subtitle;
  final IconData? icon;
  final Widget? leading;
  final VoidCallback? onTap;
  final EdgeInsets? padding;

  const InfoRow({
    this.title,
    this.subtitle,
    this.leading,
    this.icon,
    this.onTap,
    this.padding,
    super.key,
  })  : assert(leading == null || icon == null, 'You cannot provide a leading widget and an icon'),
        assert(leading != null || icon != null, 'Provide a leading widget or icon');

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: onTap != null,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: padding ?? const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
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
                      DefaultTextStyle(
                        style: context.textTheme.titleMedium!,
                        child: title!,
                      ),
                    if (subtitle != null)
                      DefaultTextStyle(
                        style: context.textTheme.bodyMedium!,
                        child: subtitle!,
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
      ),
    );
  }
}
