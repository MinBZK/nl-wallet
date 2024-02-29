import 'package:flutter/material.dart';

import '../../../../environment.dart';
import '../../../util/extension/build_context_extension.dart';

class MockIndicatorText extends StatelessWidget {
  final TextStyle? textStyle;

  const MockIndicatorText({this.textStyle, super.key});

  @override
  Widget build(BuildContext context) {
    if (!Environment.mockRepositories) return const SizedBox.shrink();
    return Container(
      decoration: BoxDecoration(
        color: context.colorScheme.errorContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      margin: const EdgeInsets.symmetric(vertical: 8),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      child: Text(
        'Mock build',
        style: (textStyle ?? context.textTheme.bodyMedium)?.copyWith(color: context.colorScheme.onErrorContainer),
      ),
    );
  }
}
