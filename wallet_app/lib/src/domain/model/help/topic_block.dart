import 'package:equatable/equatable.dart';

sealed class TopicBlock extends Equatable {
  const TopicBlock();
}

/// Bold subheading (renders as a semantic header).
class TopicHeadingBlock extends TopicBlock {
  final String text;

  const TopicHeadingBlock(this.text);

  @override
  List<Object?> get props => [text];
}

/// A paragraph of inline markdown. Also the fallback for any block that doesn't match another shape.
class TopicParagraphBlock extends TopicBlock {
  final String markdown;

  const TopicParagraphBlock(this.markdown);

  @override
  List<Object?> get props => [markdown];
}

/// Unordered list of items.
class TopicBulletListBlock extends TopicBlock {
  final List<String> items;

  const TopicBulletListBlock(this.items);

  @override
  List<Object?> get props => [items];
}

/// "See also" block: pointer to related topics.
class TopicReferenceBlock extends TopicBlock {
  final List<TopicReferenceLink> links;

  const TopicReferenceBlock(this.links);

  @override
  List<Object?> get props => [links];
}

/// A single link inside a [TopicReferenceBlock].
class TopicReferenceLink extends Equatable {
  final String label;
  final String topicId;

  const TopicReferenceLink({
    required this.label,
    required this.topicId,
  });

  @override
  List<Object?> get props => [label, topicId];
}
