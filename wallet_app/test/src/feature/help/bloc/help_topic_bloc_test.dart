import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/help/help_category.dart';
import 'package:wallet/src/domain/model/help/topic_block.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/help/bloc/help_topic_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  const locale = Locale('en');
  const testCid = 'what_is_wallet';

  late MockGetHelpCategoriesUseCase getHelpCategoriesUseCase;
  late MockGetHelpTopicBlocksUseCase getHelpTopicBlocksUseCase;

  setUp(() {
    getHelpCategoriesUseCase = MockGetHelpCategoriesUseCase();
    getHelpTopicBlocksUseCase = MockGetHelpTopicBlocksUseCase();
    when(getHelpCategoriesUseCase.invoke(any)).thenAnswer((_) async => const Result.success(<HelpCategory>[]));
    when(getHelpTopicBlocksUseCase.invoke(any, any)).thenAnswer((_) async => const Result.success(<TopicBlock>[]));
  });

  HelpTopicBloc buildBloc() => HelpTopicBloc(getHelpCategoriesUseCase, getHelpTopicBlocksUseCase, locale);

  blocTest<HelpTopicBloc, HelpTopicState>(
    'initial state is HelpTopicInitial',
    build: buildBloc,
    verify: (bloc) => expect(bloc.state, isA<HelpTopicInitial>()),
  );

  blocTest<HelpTopicBloc, HelpTopicState>(
    'emits [InProgress, LoadSuccess] when the usecase returns blocks',
    build: () {
      when(getHelpTopicBlocksUseCase.invoke(any, any)).thenAnswer(
        (_) async => const Result.success([
          TopicParagraphBlock('NL Wallet is an app for your cards.'),
          TopicHeadingBlock('How it works'),
        ]),
      );
      return buildBloc();
    },
    act: (bloc) => bloc.add(const HelpTopicLoadTriggered(testCid)),
    expect: () => const <HelpTopicState>[
      HelpTopicLoadInProgress(),
      HelpTopicLoadSuccess(
        title: '',
        blocks: [
          TopicParagraphBlock('NL Wallet is an app for your cards.'),
          TopicHeadingBlock('How it works'),
        ],
      ),
    ],
  );

  blocTest<HelpTopicBloc, HelpTopicState>(
    'emits [InProgress, LoadFailure] when the topic blocks usecase fails',
    build: () {
      when(getHelpTopicBlocksUseCase.invoke(any, any)).thenAnswer(
        (_) async => const Result.error(GenericError('asset missing', sourceError: 'asset missing')),
      );
      return buildBloc();
    },
    act: (bloc) => bloc.add(const HelpTopicLoadTriggered(testCid)),
    expect: () => const <HelpTopicState>[
      HelpTopicLoadInProgress(),
      HelpTopicLoadFailure(GenericError('asset missing', sourceError: 'asset missing')),
    ],
  );

  blocTest<HelpTopicBloc, HelpTopicState>(
    'emits [InProgress, LoadFailure] when the categories usecase fails',
    build: () {
      when(getHelpCategoriesUseCase.invoke(any)).thenAnswer(
        (_) async => const Result.error(GenericError('unsupported locale', sourceError: 'locale')),
      );
      return buildBloc();
    },
    act: (bloc) => bloc.add(const HelpTopicLoadTriggered(testCid)),
    expect: () => const <HelpTopicState>[
      HelpTopicLoadInProgress(),
      HelpTopicLoadFailure(GenericError('unsupported locale', sourceError: 'locale')),
    ],
  );

  group('events', () {
    test('HelpTopicLoadTriggered equality by topicId', () {
      const a = HelpTopicLoadTriggered('cid_one');
      const b = HelpTopicLoadTriggered('cid_one');
      const c = HelpTopicLoadTriggered('cid_two');
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });

    test('HelpTopicLoadTriggered equality considers visitedTopicIds', () {
      const a = HelpTopicLoadTriggered('cid', visitedTopicIds: ['x']);
      const b = HelpTopicLoadTriggered('cid', visitedTopicIds: ['x']);
      const c = HelpTopicLoadTriggered('cid', visitedTopicIds: ['y']);
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('visited-topic filter', () {
    blocTest<HelpTopicBloc, HelpTopicState>(
      'removes reference links whose target is on the visited chain',
      build: () {
        when(getHelpTopicBlocksUseCase.invoke(any, any)).thenAnswer(
          (_) async => const Result.success([
            TopicParagraphBlock('body'),
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Back to X', topicId: 'topic_x'),
              TopicReferenceLink(label: 'Fresh link', topicId: 'topic_c'),
            ]),
          ]),
        );
        return buildBloc();
      },
      act: (bloc) => bloc.add(const HelpTopicLoadTriggered('topic_a', visitedTopicIds: ['topic_x'])),
      expect: () => const <HelpTopicState>[
        HelpTopicLoadInProgress(),
        HelpTopicLoadSuccess(
          title: '',
          blocks: [
            TopicParagraphBlock('body'),
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Fresh link', topicId: 'topic_c'),
            ]),
          ],
        ),
      ],
    );

    blocTest<HelpTopicBloc, HelpTopicState>(
      'drops the reference block entirely when every link is filtered',
      build: () {
        when(getHelpTopicBlocksUseCase.invoke(any, any)).thenAnswer(
          (_) async => const Result.success([
            TopicHeadingBlock('Steps'),
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Back to X', topicId: 'topic_x'),
              TopicReferenceLink(label: 'Back to A', topicId: 'topic_a'),
            ]),
          ]),
        );
        return buildBloc();
      },
      act: (bloc) => bloc.add(const HelpTopicLoadTriggered('topic_b', visitedTopicIds: ['topic_x', 'topic_a'])),
      expect: () => const <HelpTopicState>[
        HelpTopicLoadInProgress(),
        HelpTopicLoadSuccess(title: '', blocks: [TopicHeadingBlock('Steps')]),
      ],
    );

    blocTest<HelpTopicBloc, HelpTopicState>(
      'leaves blocks untouched when visited chain is empty',
      build: () {
        when(getHelpTopicBlocksUseCase.invoke(any, any)).thenAnswer(
          (_) async => const Result.success([
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Any', topicId: 'topic_any'),
            ]),
          ]),
        );
        return buildBloc();
      },
      act: (bloc) => bloc.add(const HelpTopicLoadTriggered('topic_a')),
      expect: () => const <HelpTopicState>[
        HelpTopicLoadInProgress(),
        HelpTopicLoadSuccess(
          title: '',
          blocks: [
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Any', topicId: 'topic_any'),
            ]),
          ],
        ),
      ],
    );
  });
}
