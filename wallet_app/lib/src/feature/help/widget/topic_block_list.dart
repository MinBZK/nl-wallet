import 'package:flutter/material.dart';

import '../../../domain/model/help/topic_block.dart';
import '../../../wallet_constants.dart';
import 'topic_bullet_list.dart';
import 'topic_heading.dart';
import 'topic_paragraph.dart';
import 'topic_references.dart';

const _kBlockSpacing = 16.0;

/// Renders a list of parsed [TopicBlock]s (see [TopicBlockMapper]) as a
/// column of semantically-tagged widgets.
///
/// Blocks are assumed to be already filtered for the origin-subcategory rule
/// by the surrounding bloc — this widget treats every [TopicReferenceBlock] as
/// rendered as-is.
class TopicBlockList extends StatelessWidget {
  final List<TopicBlock> blocks;

  /// Invoked when the user taps a `help://{topicId}` link (either in a
  /// reference block or an inline paragraph link). Receives the target
  /// topic id. The caller is responsible for navigation.
  final Function(String)? onReferenceTap;

  const TopicBlockList({
    required this.blocks,
    this.onReferenceTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      shrinkWrap: true,
      padding: EdgeInsets.zero,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: blocks.length,
      itemBuilder: (context, index) => _blockWidget(blocks[index]),
      separatorBuilder: (context, index) => const SizedBox(height: _kBlockSpacing),
    );
  }

  Widget _blockWidget(TopicBlock block) {
    return switch (block) {
      final TopicHeadingBlock heading => _horizontallyPadded(TopicHeading(block: heading)),
      final TopicParagraphBlock paragraph => _horizontallyPadded(
        TopicParagraph(block: paragraph, onReferenceTap: onReferenceTap),
      ),
      final TopicBulletListBlock list => _horizontallyPadded(TopicBulletList(block: list)),
      // Reference block owns its own padding — the divider must span full width.
      final TopicReferenceBlock ref => TopicReferences(block: ref, onReferenceTap: onReferenceTap),
    };
  }

  Widget _horizontallyPadded(Widget child) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: kDefaultHorizontalPadding),
      child: child,
    );
  }
}
