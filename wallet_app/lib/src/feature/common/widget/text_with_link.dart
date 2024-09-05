import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import 'focus_builder.dart';

class TextWithLink extends StatelessWidget {
  final String fullText;
  final String ctaText;
  final String? onTapHint;
  final TextStyle? style;
  final TextAlign textAlign;
  final VoidCallback? onCtaPressed;

  TextWithLink({
    required this.fullText,
    required this.ctaText,
    this.onTapHint,
    this.style,
    this.textAlign = TextAlign.start,
    this.onCtaPressed,
    super.key,
  })  : assert(fullText.contains(ctaText), 'ctaText should be part of the full text'),
        assert(
            kDebugMode && fullText.split(ctaText).length == 2,
            'Currently only text formatted as "View {cta} for more info" '
            'is supported (i.e. where ctaText is enclosed in the fullText)');

  @override
  Widget build(BuildContext context) {
    final textStyle = style ?? context.textTheme.bodyLarge;
    final parts = fullText.split(ctaText);

    /// Fallback for production, so we don't crash for an ill formatted text.
    if (parts.length != 2) return Text.rich(fullText.toTextSpan(context), style: textStyle);
    return FocusBuilder(
      onEnterPressed: onCtaPressed,
      builder: (context, hasFocus) {
        return Semantics(
          onTap: onCtaPressed,
          onTapHint: onTapHint,
          attributedLabel: fullText.toAttributedString(context),
          excludeSemantics: true,
          child: Text.rich(
            locale: context.activeLocale,
            TextSpan(
              style: textStyle,
              children: [
                TextSpan(text: parts.first),
                TextSpan(
                  text: ctaText,
                  style: TextStyle(
                    color: context.colorScheme.primary,
                    decoration: TextDecoration.underline,
                    backgroundColor: hasFocus ? context.theme.focusColor : null,
                  ),
                  recognizer: TapGestureRecognizer()..onTap = onCtaPressed,
                ),
                TextSpan(text: parts.last),
              ],
            ),
            textAlign: textAlign,
            textScaler: MediaQuery.textScalerOf(context),
          ),
        );
      },
    );
  }
}
