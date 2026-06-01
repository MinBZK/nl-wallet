import 'package:equatable/equatable.dart';

import 'help_topic_group.dart';

class HelpSubcategory extends Equatable {
  final String id;
  final String title;
  final List<HelpTopicGroup> groups;

  const HelpSubcategory({
    required this.id,
    required this.title,
    required this.groups,
  });

  @override
  List<Object?> get props => [id, title, groups];
}
