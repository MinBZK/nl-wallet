import 'package:flutter/material.dart';

import '../../../domain/model/help/topic_block.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/text/title_text.dart';

class TopicHeading extends StatelessWidget {
  final TopicHeadingBlock block;

  const TopicHeading({required this.block, super.key});

  @override
  Widget build(BuildContext context) {
    return TitleText(
      block.text,
      style: context.textTheme.titleSmall,
    );
  }
}
