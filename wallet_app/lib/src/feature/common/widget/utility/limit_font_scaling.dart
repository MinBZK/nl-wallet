import 'package:flutter/material.dart';

class LimitFontScaling extends StatelessWidget {
  final double maxScaleFactor;
  final Widget child;

  const LimitFontScaling({
    required this.maxScaleFactor,
    required this.child,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MediaQuery.withClampedTextScaling(
      maxScaleFactor: maxScaleFactor,
      child: child,
    );
  }
}
