import 'package:equatable/equatable.dart';

import 'help_topic.dart';

class HelpSubcategory extends Equatable {
  final String id;
  final String title;
  final List<HelpTopic> topics;

  const HelpSubcategory({
    required this.id,
    required this.title,
    required this.topics,
  });

  @override
  List<Object?> get props => [id, title, topics];
}
