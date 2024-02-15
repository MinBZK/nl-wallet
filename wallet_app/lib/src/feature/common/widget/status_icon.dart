import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

const double _kStatusIconSize = 56;

class StatusIcon extends StatelessWidget {
  final IconData icon;
  final Color? color;

  const StatusIcon({required this.icon, this.color, super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: _kStatusIconSize,
      height: _kStatusIconSize,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: color ?? context.colorScheme.primary,
      ),
      child: Icon(icon, color: Colors.white),
    );
  }
}
