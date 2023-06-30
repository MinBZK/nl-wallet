import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class PinHeader extends StatelessWidget {
  final String title;
  final String description;
  final bool hasError;

  const PinHeader({
    Key? key,
    required this.title,
    required this.description,
    required this.hasError,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: _buildHeader(context),
    );
  }

  Widget _buildHeader(BuildContext context) {
    final textColor = hasError ? context.colorScheme.error : null;
    return Column(
      crossAxisAlignment: _resolveCrossAxisAlignment(context),
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Text(
          title,
          style: context.textTheme.displayMedium?.copyWith(color: textColor),
          textAlign: _resolveTextAlignment(context),
        ),
        const SizedBox(height: 8),
        Text(
          description,
          style: context.textTheme.bodyLarge?.copyWith(color: textColor),
          textAlign: _resolveTextAlignment(context),
        ),
      ],
    );
  }

  CrossAxisAlignment _resolveCrossAxisAlignment(BuildContext context) {
    return context.isLandscape ? CrossAxisAlignment.center : CrossAxisAlignment.start;
  }

  TextAlign _resolveTextAlignment(BuildContext context) {
    return context.isLandscape ? TextAlign.center : TextAlign.start;
  }
}
