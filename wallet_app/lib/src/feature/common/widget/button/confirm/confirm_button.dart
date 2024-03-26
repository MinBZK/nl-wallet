import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';

/// The size of the icon
const _kButtonIconSize = 16.0;

/// The spacing between de icon and the text
const _kButtonIconSpacing = 8.0;

/// The default padding inside the button
const _kButtonTextHorizontalPadding = 16.0;

class ConfirmButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final String text;
  final String? semanticsLabel;
  final IconData? icon;
  final ConfirmButtonType buttonType;

  const ConfirmButton({
    required this.text,
    this.onPressed,
    this.icon = Icons.check,
    this.semanticsLabel,
    required this.buttonType,
    super.key,
  });

  const ConfirmButton.accept({
    this.onPressed,
    required this.text,
    this.semanticsLabel,
    this.icon,
    this.buttonType = ConfirmButtonType.primary,
    super.key = const Key('acceptButton'),
  });

  const ConfirmButton.reject({
    this.onPressed,
    required this.text,
    this.semanticsLabel,
    this.icon,
    this.buttonType = ConfirmButtonType.outlined,
    super.key = const Key('rejectButton'),
  });

  @override
  Widget build(BuildContext context) {
    final text = _buildButtonText(context, buttonType, this.text, semanticsLabel);
    final textAndIcon = Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Icon(icon, size: _kButtonIconSize),
        const SizedBox(width: _kButtonIconSpacing),
        Flexible(child: text),
      ],
    );
    return SizedBox(
      width: double.infinity,
      child: _buildButtonType(
        type: buttonType,
        onPressed: onPressed,
        child: icon == null ? text : textAndIcon,
      ),
    );
  }

  Widget _buildButtonType({
    required ConfirmButtonType type,
    VoidCallback? onPressed,
    required Widget child,
    Key? key,
  }) {
    final buttonStyle = ButtonStyle(
      padding: MaterialStateProperty.all<EdgeInsets>(const EdgeInsets.symmetric(vertical: 8)),
    );
    switch (type) {
      case ConfirmButtonType.primary:
        return ElevatedButton(
          style: buttonStyle,
          key: key,
          onPressed: onPressed,
          child: child,
        );
      case ConfirmButtonType.outlined:
        return OutlinedButton(
          style: buttonStyle,
          key: key,
          onPressed: onPressed,
          child: child,
        );
      case ConfirmButtonType.text:
        return TextButton(
          style: buttonStyle,
          key: key,
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
      maxLines: 1 /* single line */,
      overflow: TextOverflow.ellipsis,
      semanticsLabel: textSemanticsLabel,
      style: _getButtonTextStyle(context),
    );
  }

  TextStyle? _getButtonTextStyle(BuildContext context) {
    final states = {MaterialState.focused};
    switch (buttonType) {
      case ConfirmButtonType.primary:
        return context.theme.elevatedButtonTheme.style?.textStyle?.resolve(states);
      case ConfirmButtonType.outlined:
        return context.theme.outlinedButtonTheme.style?.textStyle?.resolve(states);
      case ConfirmButtonType.text:
        return context.theme.textButtonTheme.style?.textStyle?.resolve(states);
    }
  }

  /// Checks if the current configuration can be rendered
  /// inside [availableWidth] without line breaks.
  bool fitsOnSingleLine(BuildContext context, double availableWidth) {
    final reservedIconSpace = icon == null ? 0 : _kButtonIconSize + _kButtonIconSpacing;
    final availableWithForText = availableWidth - (_kButtonTextHorizontalPadding * 2) - reservedIconSpace;

    final TextSpan textSpan = TextSpan(text: text, style: _getButtonTextStyle(context));
    final TextPainter painter = TextPainter(
      maxLines: 1 /* single line */,
      text: textSpan,
      textAlign: TextAlign.start,
      textDirection: TextDirection.ltr,
      textScaler: context.textScaler,
    );
    painter.layout(maxWidth: availableWithForText);
    return !painter.didExceedMaxLines;
  }
}

enum ConfirmButtonType { primary, outlined, text }
