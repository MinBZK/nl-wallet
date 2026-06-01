import 'package:freezed_annotation/freezed_annotation.dart';

part 'help_topic_screen_argument.freezed.dart';
part 'help_topic_screen_argument.g.dart';

/// Navigation argument for the topic detail screen.
///
/// [visitedTopicIds] is the chain of topics the user has drilled through via
/// "Other questions" to reach this screen. It is used to filter out circular
/// see-also references — a topic link that points back to a topic already on
/// the navigation chain is hidden, to prevent the user from looping between
/// the same set of topics.
@Freezed(copyWith: false)
abstract class HelpTopicScreenArgument with _$HelpTopicScreenArgument {
  const factory HelpTopicScreenArgument({
    required String topicId,
    @Default(<String>[]) List<String> visitedTopicIds,
  }) = _HelpTopicScreenArgument;

  factory HelpTopicScreenArgument.fromJson(Map<String, dynamic> json) => _$HelpTopicScreenArgumentFromJson(json);
}
