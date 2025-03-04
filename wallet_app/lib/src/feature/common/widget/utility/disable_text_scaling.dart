import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

/// Disable text scaling for all descendants.
class DisableTextScaling extends StatelessWidget {
  final Widget child;
  final bool disableTextScaling;

  const DisableTextScaling({
    required this.child,
    this.disableTextScaling = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return MediaQuery(
      data: context.mediaQuery.copyWith(textScaler: disableTextScaling ? TextScaler.noScaling : null),
      child: child,
    );
  }
}
