import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'focus_builder.dart';

const _kHyperlinkHorizontalPadding = 2.0;

class UrlSpan extends WidgetSpan {
  @override
  PlaceholderAlignment get alignment => PlaceholderAlignment.baseline;

  @override
  TextBaseline? get baseline => TextBaseline.alphabetic;

  UrlSpan({
    required String ctaText,
    VoidCallback? onPressed,
    TextStyle? textStyle,
  }) : super(
          child: FocusBuilder(
            builder: (context, hasFocus) {
              final urlTextStyle = (textStyle ?? context.textTheme.bodyLarge)?.copyWith(
                color: context.colorScheme.primary,
                decoration: TextDecoration.underline,
                fontWeight: hasFocus ? FontWeight.bold : null,
              );
              // Border that is drawn around the url when it has focus.
              final focusedBorderDecoration = BoxDecoration(
                border: Border.all(
                  color: context.textTheme.bodyLarge?.color ?? context.colorScheme.primary,
                  strokeAlign: BorderSide.strokeAlignOutside,
                ),
              );
              return GestureDetector(
                onTap: onPressed,
                child: Container(
                  padding: EdgeInsets.symmetric(horizontal: hasFocus ? _kHyperlinkHorizontalPadding : 0),
                  decoration: hasFocus ? focusedBorderDecoration : null,
                  child: Text(
                    ctaText,
                    style: urlTextStyle,
                    textScaler: TextScaler.noScaling, // Scaled by the parent
                  ),
                ),
              );
            },
          ),
        );
}
