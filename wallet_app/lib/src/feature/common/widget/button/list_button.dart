import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'button_content.dart';

const _kButtonHeight = 76.0;
const _kLandscapeButtonHeight = 56.0;

const _kVerticalPadding = 20.0;
const _kLandscapeVerticalPadding = 10.0;

/// A Button that spans the full width of the screen and wraps the [child] with optional bottom and top dividers.
class ListButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final DividerSide dividerSide;
  final Text text;
  final Widget? icon;
  final IconPosition iconPosition;
  final MainAxisAlignment mainAxisAlignment;
  final Widget? trailing;

  const ListButton({
    required this.text,
    this.icon = const Icon(Icons.arrow_forward_outlined),
    this.onPressed,
    this.dividerSide = DividerSide.horizontal,
    this.iconPosition = IconPosition.end,
    this.mainAxisAlignment = MainAxisAlignment.start,
    this.trailing,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        if (dividerSide.top) _buildDivider(context),
        TextButton(
          style: _resolveButtonStyle(context),
          onPressed: onPressed,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Expanded(child: _buildContent()),
              trailing ?? const SizedBox.shrink(),
            ],
          ),
        ),
        if (dividerSide.bottom) _buildDivider(context),
      ],
    );
  }

  ButtonStyle _resolveButtonStyle(BuildContext context) => context.theme.textButtonTheme.style!.copyWith(
        minimumSize: WidgetStatePropertyAll(
          Size(0, context.isLandscape ? _kLandscapeButtonHeight : _kButtonHeight),
        ),
        padding: WidgetStatePropertyAll(
          EdgeInsets.symmetric(
            horizontal: 16,
            vertical: context.isLandscape ? _kLandscapeVerticalPadding : _kVerticalPadding,
          ),
        ),
        shape: const WidgetStatePropertyAll(
          RoundedRectangleBorder(borderRadius: BorderRadius.zero),
        ),
      );

  ButtonContent _buildContent() => ButtonContent(
        text: text,
        icon: icon,
        iconPosition: iconPosition,
        mainAxisAlignment: mainAxisAlignment,
      );

  Divider _buildDivider(BuildContext context) => Divider(height: context.theme.dividerTheme.thickness);
}

enum DividerSide { none, top, bottom, horizontal }

extension DividerSideExtension on DividerSide {
  bool get top => this == DividerSide.top || this == DividerSide.horizontal;

  bool get bottom => this == DividerSide.bottom || this == DividerSide.horizontal;
}
