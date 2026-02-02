import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../button/list_button.dart';

class SwitchSettingRow extends StatelessWidget {
  /// The primary content of the switch setting row.
  ///
  /// Typically a [Text] widget.
  final Widget label;

  /// Additional content displayed below the [label].
  ///
  /// Typically a [Text] widget.
  final Widget? subtitle;

  /// Called when the user toggles the switch on or off.
  final ValueChanged<bool>? onChanged;

  /// Whether this switch is on or off.
  final bool value;

  /// Specifies which sides of the setting row should have a divider.
  final DividerSide dividerSide;

  const SwitchSettingRow({
    super.key,
    required this.label,
    this.subtitle,
    required this.value,
    this.onChanged,
    this.dividerSide = DividerSide.none,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (dividerSide.top) const Divider(),
        TextButton(
          style: context.theme.textButtonTheme.style?.copyWith(
            padding: const WidgetStatePropertyAll(EdgeInsets.zero),
            shape: const WidgetStatePropertyAll(ContinuousRectangleBorder()),
          ),
          onPressed: () => onChanged?.call(!value),
          child: ConstrainedBox(
            constraints: const BoxConstraints(minHeight: 76, minWidth: double.infinity),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 16),
              child: Row(
                mainAxisSize: MainAxisSize.max,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  Expanded(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        DefaultTextStyle(
                          style: context.textTheme.titleMedium!,
                          child: label,
                        ),
                        if (subtitle != null) ...[
                          const SizedBox(height: 8),
                          DefaultTextStyle(
                            style: context.textTheme.bodyMedium!,
                            child: subtitle!,
                          ),
                        ],
                      ],
                    ),
                  ),
                  const SizedBox(width: 12),
                  Switch(
                    value: value,
                    onChanged: onChanged,
                  ),
                ],
              ),
            ),
          ),
        ),
        if (dividerSide.bottom) const Divider(),
      ],
    );
  }
}
