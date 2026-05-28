part of 'help_topic_bloc.dart';

sealed class HelpTopicState extends Equatable {
  const HelpTopicState();

  @override
  List<Object?> get props => [];
}

class HelpTopicInitial extends HelpTopicState {
  const HelpTopicInitial();
}

class HelpTopicLoadInProgress extends HelpTopicState {
  const HelpTopicLoadInProgress();
}

class HelpTopicLoadSuccess extends HelpTopicState {
  final String title;
  final List<TopicBlock> blocks;

  const HelpTopicLoadSuccess({required this.title, required this.blocks});

  @override
  List<Object?> get props => [title, blocks];
}

class HelpTopicLoadFailure extends HelpTopicState implements ErrorState {
  @override
  final ApplicationError error;

  const HelpTopicLoadFailure(this.error);

  @override
  List<Object?> get props => [error];
}
