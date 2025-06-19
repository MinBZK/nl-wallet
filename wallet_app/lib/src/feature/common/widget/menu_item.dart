import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/list_extension.dart';
import 'button/list_button.dart';
import 'default_text_and_focus_style.dart';

const kMenuItemNormalIconSize = 24.0;
const kMenuItemLargeIconSize = 40.0;
const kRightIconSize = 24.0;
const kRightIconFocusedSize = 30.0;

class MenuItem extends StatefulWidget {
  /// The main text displayed in the menu item.
  final Widget? label;

  /// The text displayed below the label.
  final Widget? subtitle;

  /// The text displayed below the subtitle, typically used for additional information.
  final Widget? underline;

  /// The widget displayed on the left side of the menu item, typically an icon.
  final Widget? leftIcon;

  /// The widget displayed on the right side of the menu item, typically a chevron icon.
  final Widget? rightIcon;

  /// The widget displayed to the left of the subtitle, typically used to indicate an error.
  final Widget? errorIcon;

  /// The callback that is called when the menu item is tapped.
  final VoidCallback? onPressed;

  /// Determines if the [leftIcon] should use a larger size (40.0) instead of the normal size (24.0).
  /// This is useful for displaying logos or larger icons.
  final bool largeIcon;

  /// Specifies which sides of the menu item should have a divider.
  final DividerSide dividerSide;

  const MenuItem({
    this.label,
    this.subtitle,
    this.underline,
    this.leftIcon,
    this.rightIcon = const Icon(Icons.chevron_right_outlined),
    this.errorIcon,
    this.onPressed,
    this.largeIcon = false,
    this.dividerSide = DividerSide.none,
    super.key,
  });

  @override
  State<MenuItem> createState() => _MenuItemState();
}

class _MenuItemState extends State<MenuItem> {
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
    final Color? textPressedColor =
        context.theme.textButtonTheme.style?.foregroundColor?.resolve({WidgetState.pressed});
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (widget.dividerSide.top) const Divider(),
        TextButton(
          style: context.theme.textButtonTheme.style?.copyWith(
            padding: const WidgetStatePropertyAll(EdgeInsets.zero),
            shape: const WidgetStatePropertyAll(RoundedRectangleBorder(borderRadius: BorderRadius.zero)),
          ),
          statesController: _statesController,
          onPressed: widget.onPressed,
          child: ConstrainedBox(
            constraints: const BoxConstraints(minHeight: 80, minWidth: double.infinity),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 16),
              child: Row(
                mainAxisSize: MainAxisSize.max,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  SizedBox(width: widget.leftIcon == null ? 0 : 16),
                  if (widget.leftIcon != null) _buildLeftIcon(),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        if (widget.label != null)
                          DefaultTextAndFocusStyle(
                            textStyle: context.textTheme.titleMedium!,
                            statesController: _statesController,
                            pressedOrFocusedColor: textPressedColor,
                            child: widget.label!,
                          ),
                        if (widget.subtitle != null)
                          Row(
                            children: [
                              if (widget.errorIcon != null) ...[
                                SizedBox(
                                  height: 16,
                                  width: 16,
                                  child: IconTheme(
                                    data: IconThemeData(color: context.colorScheme.error, size: 16),
                                    child: widget.errorIcon!,
                                  ),
                                ),
                                const SizedBox(width: 8),
                              ],
                              Expanded(
                                child: DefaultTextAndFocusStyle(
                                  textStyle: context.textTheme.bodyMedium!,
                                  statesController: _statesController,
                                  pressedOrFocusedColor: textPressedColor,
                                  child: widget.subtitle!,
                                ),
                              ),
                            ],
                          ),
                        if (widget.underline != null)
                          DefaultTextAndFocusStyle(
                            textStyle: context.textTheme.bodySmall!,
                            statesController: _statesController,
                            pressedOrFocusedColor: textPressedColor,
                            child: widget.underline!,
                          ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 16),
                  if (widget.rightIcon != null) _buildRightIcon(),
                  SizedBox(width: widget.rightIcon == null ? 0 : 8),
                ].nonNullsList,
              ),
            ),
          ),
        ),
        if (widget.dividerSide.bottom) const Divider(),
      ],
    );
  }

  Widget _buildLeftIcon() {
    assert(widget.leftIcon != null, 'leftIcon is expected to exist');
    return SizedBox(
      width: widget.largeIcon ? kMenuItemLargeIconSize : kMenuItemNormalIconSize,
      height: widget.largeIcon ? kMenuItemLargeIconSize : kMenuItemNormalIconSize,
      child: IconTheme(
        data: IconThemeData(
          size: kMenuItemNormalIconSize,
          color: context.theme.iconTheme.color,
        ),
        child: widget.leftIcon!,
      ),
    );
  }

  Widget _buildRightIcon() {
    assert(widget.rightIcon != null, 'rightIcon is expected to exist');
    return SizedBox(
      width: 48,
      height: 48,
      child: IconTheme(
        data: IconThemeData(
          size: _statesController.value.isPressedOrFocused ? kRightIconFocusedSize : kRightIconSize,
          color: context.theme.iconButtonTheme.style?.iconColor?.resolve(_statesController.value),
        ),
        child: widget.rightIcon!,
      ),
    );
  }
}
