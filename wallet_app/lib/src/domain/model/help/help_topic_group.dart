import 'package:equatable/equatable.dart';

import 'help_topic.dart';

/// The two recognized group kinds inside a help subcategory. The YAML manifest
/// stores them as `groupId: help` / `groupId: information`; the repository
/// parser converts the raw string to this enum and drops groups with an
/// unknown id.
enum HelpTopicGroupKind { help, information }

class HelpTopicGroup extends Equatable {
  final HelpTopicGroupKind kind;
  final List<HelpTopic> topics;

  const HelpTopicGroup({
    required this.kind,
    required this.topics,
  });

  @override
  List<Object?> get props => [kind, topics];
}
