import 'dart:ui';

import '../../../../data/repository/help/help_content_repository.dart';
import '../../../model/help/topic_block.dart';
import '../../../model/result/result.dart';
import '../get_help_topic_blocks_usecase.dart';

class GetHelpTopicBlocksUseCaseImpl extends GetHelpTopicBlocksUseCase {
  final HelpContentRepository _helpContentRepository;

  GetHelpTopicBlocksUseCaseImpl(this._helpContentRepository);

  @override
  Future<Result<List<TopicBlock>>> invoke(String topicId, Locale locale) {
    return tryCatch(
      () => _helpContentRepository.getTopicBlocks(topicId, locale),
      'Failed to load help topic blocks for $topicId',
    );
  }
}
