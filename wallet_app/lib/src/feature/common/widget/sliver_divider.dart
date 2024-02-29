import 'package:flutter/material.dart';

class SliverDivider extends StatelessWidget {
  final double? indent, endIndent, height;

  const SliverDivider({
    this.indent = 0,
    this.endIndent = 0,
    this.height = 1,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SliverToBoxAdapter(
      child: Divider(
        height: height,
        indent: indent,
        endIndent: endIndent,
      ),
    );
  }
}
