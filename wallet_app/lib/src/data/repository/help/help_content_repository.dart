import 'dart:ui';

import '../../../domain/model/help/help_category.dart';
import '../../../domain/model/help/topic_block.dart';

abstract class HelpContentRepository {
  Future<List<HelpCategory>> getCategories(Locale locale);

  Future<List<TopicBlock>> getTopicBlocks(String topicId, Locale locale);
}
