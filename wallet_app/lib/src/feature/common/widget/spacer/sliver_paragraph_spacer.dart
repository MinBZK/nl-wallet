import 'package:flutter/material.dart';

import 'paragraph_spacer.dart';

class SliverParagraphSpacer extends StatelessWidget {
  const SliverParagraphSpacer({super.key});

  @override
  Widget build(BuildContext context) {
    return SliverToBoxAdapter(
      child: ParagraphSpacer(),
    );
  }
}
