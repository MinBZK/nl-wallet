import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import 'url_span.dart';

class TextWithLink extends StatelessWidget {
  final String fullText;
  final String linkText;
  final String? onTapHint;
  final TextStyle? style;
  final TextAlign textAlign;
  final VoidCallback onLinkPressed;

  TextWithLink({
    required this.fullText,
    required this.linkText,
    this.onTapHint,
    this.style,
    this.textAlign = TextAlign.start,
    required this.onLinkPressed,
    super.key,
  })  : assert(fullText.contains(linkText), 'linkText should be part of the full text'),
        assert(
            kDebugMode && fullText.split(linkText).length == 2,
            'Currently only text formatted as "View {cta} for more info" '
            'is supported (i.e. where linkText is enclosed in the fullText)');

  @override
  Widget build(BuildContext context) {
    final textStyle = style ?? context.textTheme.bodyLarge;
    final parts = fullText.split(linkText);

    /// Fallback for production, so we don't crash for an ill formatted text.
    if (parts.length != 2) return Text.rich(fullText.toTextSpan(context), style: textStyle);
    return MergeSemantics(
      child: Semantics(
        onTap: onLinkPressed,
        onTapHint: onTapHint,
        attributedLabel: fullText.toAttributedString(context),
        excludeSemantics: true,
        link: true,
        child: Text.rich(
          locale: context.activeLocale,
          TextSpan(
            style: textStyle,
            children: [
              TextSpan(text: parts.first),
              UrlSpan(
                ctaText: linkText,
                onPressed: onLinkPressed,
                textStyle: style,
              ),
              TextSpan(text: parts.last),
            ],
          ),
          textAlign: textAlign,
          textScaler: MediaQuery.textScalerOf(context),
        ),
      ),
    );
  }
}
