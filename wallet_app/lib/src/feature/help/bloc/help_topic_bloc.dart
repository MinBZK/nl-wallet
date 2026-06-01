import 'dart:ui';

import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/bloc/error_state.dart';
import '../../../domain/model/help/topic_block.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../domain/usecase/help/get_help_categories_usecase.dart';
import '../../../domain/usecase/help/get_help_topic_blocks_usecase.dart';
import '../extension/help_categories_extension.dart';

part 'help_topic_event.dart';
part 'help_topic_state.dart';

class HelpTopicBloc extends Bloc<HelpTopicEvent, HelpTopicState> {
  final GetHelpCategoriesUseCase getHelpCategoriesUseCase;
  final GetHelpTopicBlocksUseCase getHelpTopicBlocksUseCase;
  final Locale locale;

  HelpTopicBloc(
    this.getHelpCategoriesUseCase,
    this.getHelpTopicBlocksUseCase,
    this.locale,
  ) : super(const HelpTopicInitial()) {
    on<HelpTopicLoadTriggered>(_onLoadTriggered);
  }

  Future<void> _onLoadTriggered(HelpTopicLoadTriggered event, Emitter<HelpTopicState> emit) async {
    emit(const HelpTopicLoadInProgress());
    // getCategories is cached by the repository, so resolving the title here is cheap.
    final categoriesResult = await getHelpCategoriesUseCase.invoke(locale);
    final blocksResult = await getHelpTopicBlocksUseCase.invoke(event.topicId, locale);

    if (categoriesResult.hasError || blocksResult.hasError) {
      emit(HelpTopicLoadFailure((categoriesResult.error ?? blocksResult.error)!));
    } else {
      emit(
        HelpTopicLoadSuccess(
          title: categoriesResult.value!.findTopicTitle(event.topicId) ?? '',
          blocks: _filterVisited(blocksResult.value!, event.visitedTopicIds),
        ),
      );
    }
  }

  /// Removes any [TopicReferenceBlock] link whose target is already on the
  /// navigation chain that led here. If a reference block loses all its
  /// links as a result, the block itself is dropped.
  List<TopicBlock> _filterVisited(List<TopicBlock> blocks, List<String> visitedTopicIds) {
    if (visitedTopicIds.isEmpty) return blocks;
    final visited = visitedTopicIds.toSet();
    final result = <TopicBlock>[];
    for (final block in blocks) {
      if (block is TopicReferenceBlock) {
        final kept = block.links.whereNot((link) => visited.contains(link.topicId)).toList();
        if (kept.isEmpty) continue;
        result.add(TopicReferenceBlock(kept));
      } else {
        result.add(block);
      }
    }
    return result;
  }
}
