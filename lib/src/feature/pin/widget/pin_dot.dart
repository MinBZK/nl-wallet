import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';

const double _kDotSize = 16;

class PinDot extends StatelessWidget {
  final bool checked;
  final Color color;

  const PinDot({required this.checked, this.color = Colors.black, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      margin: const EdgeInsets.all(8),
      duration: kDefaultAnimationDuration,
      width: _kDotSize,
      height: _kDotSize,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: checked ? color : Colors.transparent,
        border: Border.all(color: color, width: 2, strokeAlign: StrokeAlign.inside),
      ),
    );
  }
}
