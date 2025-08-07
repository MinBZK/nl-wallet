import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class BulletListDot extends StatelessWidget {
  final Color? color;

  const BulletListDot({this.color, super.key});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Container(
        height: 4,
        width: 4,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: color ?? context.theme.iconTheme.color,
        ),
      ),
    );
  }
}
