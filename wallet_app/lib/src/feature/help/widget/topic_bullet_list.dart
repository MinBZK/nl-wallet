import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:material_ui/material_ui.dart';

import '../../../domain/model/help/topic_block.dart';
import '../../../util/extension/build_context_extension.dart';
import 'topic_markdown_style.dart';

class TopicBulletList extends StatelessWidget {
  final TopicBulletListBlock block;

  const TopicBulletList({required this.block, super.key});

  @override
  Widget build(BuildContext context) {
    final items = block.items;
    return ListView.separated(
      shrinkWrap: true,
      padding: EdgeInsets.zero,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: items.length,
      itemBuilder: (context, index) => Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ExcludeSemantics(child: Text('• ', style: context.textTheme.bodyLarge)),
          Expanded(
            // flutter_markdown_plus reads the legacy flutter/material theme.
            // TODO(Rubin): remove the bridge once it supports material_ui, see:
            // https://github.com/foresightmobile/flutter_markdown_plus/issues/144
            // https://github.com/foresightmobile/flutter_markdown_plus/pull/145
            // https://github.com/foresightmobile/flutter_markdown_plus/pull/150
            // ignore: deprecated_member_use
            child: MaterialUiCompatibilityBridge(
              child: MarkdownBody(data: items[index], styleSheet: topicMarkdownStyleSheet(context)),
            ),
          ),
        ],
      ),
      separatorBuilder: (context, index) => const SizedBox(height: 4),
    );
  }
}
