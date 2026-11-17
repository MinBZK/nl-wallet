import 'package:material_ui/material_ui.dart';

import 'paragraph_spacer.dart';

class SliverParagraphSpacer extends StatelessWidget {
  const SliverParagraphSpacer({super.key});

  @override
  Widget build(BuildContext context) {
    return const SliverToBoxAdapter(
      child: ParagraphSpacer(),
    );
  }
}
