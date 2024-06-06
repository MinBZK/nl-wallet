import 'package:flutter/material.dart';

class WalletScrollbar extends StatelessWidget {
  final Widget child;
  final ScrollController? controller;

  const WalletScrollbar({required this.child, this.controller, super.key});

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      controller: controller,
      thumbVisibility: true,
      trackVisibility: true,
      child: child,
    );
  }
}
