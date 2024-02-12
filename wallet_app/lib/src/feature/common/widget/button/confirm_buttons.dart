import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

const _kHorizontalPadding = 16.0;
const _kVerticalPadding = 24.0;

const _kButtonHeight = 48.0;
const _kButtonSpacing = 12.0;
const _kButtonTextHorizontalPadding = 16.0;
const _kButtonTextMaxLines = 1;

const _kButtonIconSize = 16.0;
const _kButtonIconSpacing = 8.0;
const _kButtonIconHorizontalSpace = _kButtonIconSize + _kButtonIconSpacing;

class ConfirmButtons extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final String acceptText;
  final String declineText;
  final String? declineTextSemanticsLabel;
  final IconData? acceptIcon;
  final IconData? declineIcon;
  final bool forceVertical;

  const ConfirmButtons({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.acceptText,
    required this.declineText,
    this.declineTextSemanticsLabel,
    this.forceVertical = false,
    this.acceptIcon = Icons.check,
    this.declineIcon = Icons.not_interested,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final bool buttonExceedsWidth = _isExceedingMaxWidth(
          context,
          ConfirmButtonType.reject,
          declineIcon != null,
          declineText,
        ) ||
        _isExceedingMaxWidth(
          context,
          ConfirmButtonType.accept,
          acceptIcon != null,
          acceptText,
        );

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: _kHorizontalPadding, vertical: _kVerticalPadding),
      child: _buildDirectionalButtonsLayout(
        context,
        vertical: forceVertical || buttonExceedsWidth,
      ),
    );
  }

  /// Aligns buttons either vertical or horizontal based on [vertical]
  Widget _buildDirectionalButtonsLayout(BuildContext context, {required bool vertical}) {
    List<Widget> children = _buildButtons(context, vertical: vertical);
    if (vertical) {
      return Column(
        mainAxisSize: MainAxisSize.min,
        children: children,
      );
    } else {
      return Row(
        children: children,
      );
    }
  }

  /// Returns buttons (including spacing) in right order depending on [buttonsExceedsWidth]
  List<Widget> _buildButtons(BuildContext context, {required bool vertical}) {
    List<Widget> children = [
      _buildButtonWrapper(
        vertical: vertical,
        child: _buildButtonContainer(
          context,
          ConfirmButtonType.reject,
          onDeclinePressed,
          declineIcon,
          declineText,
          declineTextSemanticsLabel,
        ),
      ),
      const SizedBox(width: _kButtonSpacing, height: _kButtonSpacing),
      _buildButtonWrapper(
        vertical: vertical,
        child: _buildButtonContainer(
          context,
          ConfirmButtonType.accept,
          onAcceptPressed,
          acceptIcon,
          acceptText,
          null,
        ),
      ),
    ];

    return vertical ? children.reversed.toList() : children;
  }

  /// Adds no additional Widget wrap when [vertical] is `true`
  /// Adds `Expanded` in other case; to fill available width.
  Widget _buildButtonWrapper({required bool vertical, required Widget child}) {
    if (vertical) {
      return child;
    } else {
      return Expanded(child: child);
    }
  }

  Widget _buildButtonContainer(
    BuildContext context,
    ConfirmButtonType type,
    VoidCallback onPressed,
    IconData? icon,
    String text,
    String? textSemanticsLabel,
  ) {
    return SizedBox(
      height: _kButtonHeight,
      width: double.infinity,
      child: _buildButtonType(
        type: type,
        onPressed: onPressed,
        child: icon == null
            ? _buildButtonText(context, type, text, textSemanticsLabel)
            : Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(icon, size: _kButtonIconSize),
                  const SizedBox(width: _kButtonIconSpacing),
                  Flexible(
                    child: _buildButtonText(context, type, text, textSemanticsLabel),
                  ),
                ],
              ),
      ),
    );
  }

  Widget _buildButtonType({required ConfirmButtonType type, required VoidCallback onPressed, required Widget child}) {
    switch (type) {
      case ConfirmButtonType.accept:
        return ElevatedButton(
          key: const Key('acceptButton'),
          onPressed: onPressed,
          child: child,
        );
      case ConfirmButtonType.reject:
        return OutlinedButton(
          key: const Key('rejectButton'),
          onPressed: onPressed,
          child: child,
        );
    }
  }

  Widget _buildButtonText(
    BuildContext context,
    ConfirmButtonType type,
    String text,
    String? textSemanticsLabel,
  ) {
    return Text(
      text,
      maxLines: _kButtonTextMaxLines,
      overflow: TextOverflow.ellipsis,
      semanticsLabel: textSemanticsLabel,
      style: _getButtonTextStyle(context, type),
    );
  }

  TextStyle? _getButtonTextStyle(BuildContext context, ConfirmButtonType type) {
    final states = {MaterialState.focused};
    switch (type) {
      case ConfirmButtonType.accept:
        return context.theme.elevatedButtonTheme.style?.textStyle?.resolve(states);
      case ConfirmButtonType.reject:
        return context.theme.outlinedButtonTheme.style?.textStyle?.resolve(states);
    }
  }

  bool _isExceedingMaxWidth(BuildContext context, ConfirmButtonType type, bool hasIcon, String text) {
    final double screenWidth = context.mediaQuery.size.width.roundToDouble();
    final double buttonWidth = (screenWidth - (_kHorizontalPadding * 2) - _kButtonSpacing) / 2;
    final double buttonContentWidth = buttonWidth - (_kButtonTextHorizontalPadding * 2);
    final double buttonTextMaxWidth = buttonContentWidth - (hasIcon ? _kButtonIconHorizontalSpace : 0);

    if (buttonContentWidth.isNegative || buttonTextMaxWidth.isNegative) return true;

    final TextSpan textSpan = TextSpan(
      text: text,
      style: _getButtonTextStyle(context, type),
    );

    final TextPainter painter = TextPainter(
      maxLines: _kButtonTextMaxLines,
      text: textSpan,
      textAlign: TextAlign.start,
      textDirection: TextDirection.ltr,
      textScaler: context.textScaler,
    );

    painter.layout(maxWidth: buttonTextMaxWidth);

    return painter.didExceedMaxLines;
  }
}

enum ConfirmButtonType { accept, reject }
