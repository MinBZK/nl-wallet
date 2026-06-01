import 'dart:ui';

import '../../model/help/topic_block.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetHelpTopicBlocksUseCase extends WalletUseCase {
  Future<Result<List<TopicBlock>>> invoke(String topicId, Locale locale);
}
