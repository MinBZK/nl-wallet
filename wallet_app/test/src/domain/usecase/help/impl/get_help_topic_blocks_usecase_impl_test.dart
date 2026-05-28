import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/help/topic_block.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/help/impl/get_help_topic_blocks_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockHelpContentRepository mockHelpContentRepository;
  late GetHelpTopicBlocksUseCaseImpl useCase;

  const locale = Locale('en');
  const topicId = 'what_is_wallet';

  setUp(() {
    mockHelpContentRepository = MockHelpContentRepository();
    useCase = GetHelpTopicBlocksUseCaseImpl(mockHelpContentRepository);
  });

  test('invoke returns Success with the topic blocks from the repository', () async {
    const blocks = [
      TopicParagraphBlock('NL Wallet is an app for your cards.'),
      TopicHeadingBlock('How it works'),
    ];
    when(mockHelpContentRepository.getTopicBlocks(topicId, locale)).thenAnswer((_) async => blocks);

    final result = await useCase.invoke(topicId, locale);

    expect(result, isA<Success<List<TopicBlock>>>());
    expect(result.value, blocks);
  });

  test('invoke returns an Error when the repository throws', () async {
    when(mockHelpContentRepository.getTopicBlocks(topicId, locale)).thenThrow(Exception('asset missing'));

    final result = await useCase.invoke(topicId, locale);

    expect(result, isA<Error<List<TopicBlock>>>());
    expect(result.error, isA<GenericError>());
  });
}
