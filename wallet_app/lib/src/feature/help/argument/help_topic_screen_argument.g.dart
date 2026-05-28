// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'help_topic_screen_argument.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_HelpTopicScreenArgument _$HelpTopicScreenArgumentFromJson(
  Map<String, dynamic> json,
) => _HelpTopicScreenArgument(
  topicId: json['topicId'] as String,
  visitedTopicIds: (json['visitedTopicIds'] as List<dynamic>?)?.map((e) => e as String).toList() ?? const <String>[],
);

Map<String, dynamic> _$HelpTopicScreenArgumentToJson(
  _HelpTopicScreenArgument instance,
) => <String, dynamic>{
  'topicId': instance.topicId,
  'visitedTopicIds': instance.visitedTopicIds,
};
