import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/object_extension.dart';

const _kIconSize = 16.0;
const _kSpacing = 8.0;

/// Widget that horizontally renders the icon (optional) and text as:
/// [icon][spacing][text]
/// Mainly used to consistently render content for the default buttons
/// [PrimaryButton], [SecondaryButton] & [TertiaryButton]
class ButtonContent extends StatelessWidget {
  final Text text;
  final Widget? icon;
  final IconPosition iconPosition;
  final MainAxisAlignment mainAxisAlignment;

  const ButtonContent({
    required this.text,
    this.icon,
    this.iconPosition = IconPosition.start,
    this.mainAxisAlignment = MainAxisAlignment.center,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (icon == null) return text;

    final children = switch (iconPosition) {
      IconPosition.start => [
          SizedBox(height: _kIconSize, width: _kIconSize, child: icon),
          const SizedBox(width: _kSpacing),
          Flexible(child: text), //text
        ],
      IconPosition.end => [
          Flexible(child: text),
          const SizedBox(width: _kSpacing),
          SizedBox(height: _kIconSize, width: _kIconSize, child: icon),
        ],
    };

    return Row(
      mainAxisAlignment: mainAxisAlignment,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: children,
    );
  }

  double contentWidth(BuildContext context, TextStyle style) {
    final iconWidth = icon == null ? 0 : _kIconSize + _kSpacing;
    // When a [Text.rich(..)] widget is provided the text.data does not return the content, in that case we extract it manually.
    final textContent = text.textSpan?.let((it) => it.toPlainText()) ?? text.data;
    final TextSpan textSpan = TextSpan(text: textContent, style: style);
    final TextPainter painter = TextPainter(
      maxLines: 1 /* single line */,
      text: textSpan,
      textAlign: TextAlign.start,
      textDirection: TextDirection.ltr,
      textScaler: context.textScaler,
    );
    painter.layout();
    return painter.width + iconWidth;
  }
}

enum IconPosition { start, end }
