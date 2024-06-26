import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class WalletScrollbar extends StatelessWidget {
  final Widget child;
  final ScrollController? controller;

  const WalletScrollbar({required this.child, this.controller, super.key});

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      controller: controller,
      thumbVisibility: context.theme.scrollbarTheme.thumbVisibility?.resolve({}) ?? true,
      trackVisibility: context.theme.scrollbarTheme.trackVisibility?.resolve({}) ?? false,
      child: child,
    );
  }
}
