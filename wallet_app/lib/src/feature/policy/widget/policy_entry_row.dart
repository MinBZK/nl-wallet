import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../model/policy_entry.dart';

class PolicyEntryRow extends StatelessWidget {
  final IconData? icon;
  final Widget title;
  final Widget description;
  final VoidCallback? semanticsOnTap;
  final String? semanticOnTapHint;

  const PolicyEntryRow({
    this.icon,
    required this.title,
    required this.description,
    this.semanticsOnTap,
    this.semanticOnTapHint,
    super.key,
  });

  factory PolicyEntryRow.fromPolicyEntry(PolicyEntry entry) {
    return PolicyEntryRow(
      icon: entry.icon,
      title: Text.rich(
        entry.title,
        semanticsLabel: entry.titleSemanticsLabel,
      ),
      description: Builder(
        builder: (context) {
          return Semantics(
            excludeSemantics: true,
            attributedLabel: entry.descriptionSemanticsLabel?.toAttributedString(context),
            child: Text.rich(
              entry.description,
              semanticsLabel: entry.descriptionSemanticsLabel,
            ),
          );
        },
      ),
      semanticsOnTap: entry.semanticOnTap,
      semanticOnTapHint: entry.semanticOnTapHint,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Semantics(
      link: semanticsOnTap != null,
      onTap: semanticsOnTap,
      onTapHint: semanticOnTapHint,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            icon == null
                ? const SizedBox.shrink()
                : Icon(
                    icon,
                    size: 24,
                    color: context.colorScheme.onSurfaceVariant,
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
      ),
    );
  }
}
