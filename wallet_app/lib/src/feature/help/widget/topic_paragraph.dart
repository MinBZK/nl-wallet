import 'package:flutter/material.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';

import '../../../domain/model/help/topic_block.dart';
import '../../../util/launch_util.dart';
import 'topic_markdown_style.dart';

const _kHelpUriScheme = 'help';

class TopicParagraph extends StatelessWidget {
  final TopicParagraphBlock block;
  final Function(String)? onReferenceTap;

  const TopicParagraph({
    required this.block,
    required this.onReferenceTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return MarkdownBody(
      data: block.markdown,
      onTapLink: (text, href, title) => _handleMarkdownLinkTap(href),
      styleSheet: topicMarkdownStyleSheet(context),
    );
  }

  void _handleMarkdownLinkTap(String? href) {
    if (href == null) return;
    final uri = Uri.tryParse(href);
    if (uri?.scheme == _kHelpUriScheme) {
      final topicId = uri!.host.isNotEmpty ? uri.host : uri.path;
      if (topicId.isNotEmpty) onReferenceTap?.call(topicId);
      return;
    }
    launchUrlStringCatching(href);
  }
}
