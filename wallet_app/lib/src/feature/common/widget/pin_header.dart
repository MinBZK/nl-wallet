import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'text/body_text.dart';
import 'text/title_text.dart';

class PinHeader extends StatelessWidget {
  final String title;
  final String? description;

  /// When [hasError] is true, text is rendered in [ColorScheme.error] color
  final bool hasError;

  final TextAlign textAlign;
  final Alignment? contentAlignment;

  const PinHeader({
    super.key,
    required this.title,
    this.description,
    this.textAlign = TextAlign.start,
    this.contentAlignment,
    this.hasError = false,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: _resolvePadding(context),
      alignment: _resolveAlignment(context),
      child: _buildContent(context),
    );
  }

  EdgeInsets _resolvePadding(BuildContext context) {
    return context.isLandscape
        ? const EdgeInsets.symmetric(horizontal: 24, vertical: 24)
        : const EdgeInsets.symmetric(horizontal: 16, vertical: 24);
  }

  Alignment _resolveAlignment(BuildContext context) {
    return contentAlignment ?? (context.isLandscape ? Alignment.centerLeft : Alignment.topLeft);
  }

  Widget _buildContent(BuildContext context) {
    final textColor = hasError ? context.colorScheme.error : null;
    if (description == null) {
      return TitleText(
        title,
        textAlign: textAlign,
      );
    } else {
      return Column(
        crossAxisAlignment: _resolveCrossAxisAlignment(context),
        mainAxisSize: MainAxisSize.min,
        children: [
          TitleText(
            title,
            style: context.textTheme.headlineMedium?.copyWith(color: textColor),
            textAlign: textAlign,
          ),
          const SizedBox(height: 8),
          BodyText(
            description!,
            style: context.textTheme.bodyLarge?.copyWith(color: textColor),
            textAlign: textAlign,
          ),
        ],
      );
    }
  }

  /// Select most relevant [CrossAxisAlignment] based on the provided [Alignment]
  CrossAxisAlignment _resolveCrossAxisAlignment(BuildContext context) {
    final alignment = _resolveAlignment(context);
    switch (alignment.x) {
      case 0.0:
        return CrossAxisAlignment.center;
      case -1.0:
        return CrossAxisAlignment.start;
      case 1.0:
        return CrossAxisAlignment.end;
    }
    return CrossAxisAlignment.start;
  }
}
