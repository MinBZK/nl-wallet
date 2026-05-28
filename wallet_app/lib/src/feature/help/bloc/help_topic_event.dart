part of 'help_topic_bloc.dart';

abstract class HelpTopicEvent extends Equatable {
  const HelpTopicEvent();

  @override
  List<Object?> get props => [];
}

class HelpTopicLoadTriggered extends HelpTopicEvent {
  final String topicId;

  /// Topics already visited on the drill-down chain leading to this screen.
  /// Used to filter circular see-also references from [TopicReferenceBlock]s.
  final List<String> visitedTopicIds;

  const HelpTopicLoadTriggered(this.topicId, {this.visitedTopicIds = const []});

  @override
  List<Object?> get props => [topicId, visitedTopicIds];
}
