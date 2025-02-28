import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import 'default_text_and_focus_style.dart';

const _kIconSize = 24.0;

class InfoRow extends StatefulWidget {
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
  State<InfoRow> createState() => _InfoRowState();
}

class _InfoRowState extends State<InfoRow> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(() => setState(() {})));
  }

  @override
  void dispose() {
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final textPressedColor = context.theme.textButtonTheme.style?.foregroundColor?.resolve({WidgetState.pressed});
    return Semantics(
      button: widget.onTap != null,
      child: TextButton.icon(
        statesController: _statesController,
        onPressed: widget.onTap,
        icon: _buildIcon(context),
        iconAlignment: IconAlignment.end,
        style: context.theme.iconButtonTheme.style?.copyWith(
          foregroundColor: WidgetStateProperty.resolveWith(
            // Only override the color when the button is not pressed or focused
            (states) => states.isPressedOrFocused ? null : context.colorScheme.onSurface,
          ),
          shape: const WidgetStatePropertyAll(
            RoundedRectangleBorder(borderRadius: BorderRadius.zero),
          ),
        ),
        label: Padding(
          padding: widget.padding ?? const EdgeInsets.symmetric(horizontal: 0, vertical: 24),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              widget.leading ??
                  Icon(
                    widget.icon,
                    color: context.colorScheme.onSurfaceVariant,
                    size: _kIconSize,
                  ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (widget.title != null)
                      DefaultTextAndFocusStyle(
                        statesController: _statesController,
                        textStyle: context.textTheme.titleMedium,
                        pressedOrFocusedColor: textPressedColor,
                        child: widget.title!,
                      ),
                    if (widget.subtitle != null)
                      DefaultTextAndFocusStyle(
                        statesController: _statesController,
                        textStyle: context.textTheme.bodyMedium,
                        pressedOrFocusedColor: textPressedColor,
                        child: widget.subtitle!,
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget? _buildIcon(BuildContext context) {
    if (widget.onTap == null) return null;
    return const Icon(Icons.chevron_right);
  }
}
