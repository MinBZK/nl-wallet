import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'loading_indicator.dart';

class CenteredLoadingIndicator extends StatelessWidget {
  final bool showCircularBackground;

  const CenteredLoadingIndicator({this.showCircularBackground = false, super.key});

  @override
  Widget build(BuildContext context) {
    if (showCircularBackground) {
      return Center(
        child: Container(
          decoration: BoxDecoration(shape: .circle, color: context.theme.colorScheme.surface),
          padding: const EdgeInsets.all(16),
          child: const LoadingIndicator(),
        ),
      );
    } else {
      return const Center(child: LoadingIndicator());
    }
  }
}
