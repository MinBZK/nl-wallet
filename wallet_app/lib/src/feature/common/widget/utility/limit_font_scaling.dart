import 'package:flutter/material.dart';

class LimitFontScaling extends StatelessWidget {
  final double maxScaleFactor;
  final Widget child;

  const LimitFontScaling({
    required this.maxScaleFactor,
    required this.child,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return MediaQuery.withClampedTextScaling(
      maxScaleFactor: maxScaleFactor,
      child: child,
    );
  }
}
