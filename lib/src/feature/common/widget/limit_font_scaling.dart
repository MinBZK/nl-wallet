import 'dart:math';

import 'package:flutter/material.dart';

class LimitFontScaling extends StatelessWidget {
  final double? maxTextScaleFactor;
  final Widget child;

  const LimitFontScaling({
    this.maxTextScaleFactor,
    required this.child,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final mediaQuery = MediaQuery.of(context);
    final textScaleFactor = min(maxTextScaleFactor ?? double.maxFinite, mediaQuery.textScaleFactor);
    return MediaQuery(
      data: mediaQuery.copyWith(textScaleFactor: textScaleFactor),
      child: child,
    );
  }
}
