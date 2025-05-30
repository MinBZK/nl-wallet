import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'spacer/paragraph_spacer.dart';
import 'text/body_text.dart';

/// Renders the provided [paragraphs] as [BodyText] with a 8dp separator
/// Check out the [ParagraphedList] for use without slivers.
class ParagraphedSliverList extends StatelessWidget {
  final List<String> paragraphs;

  const ParagraphedSliverList({required this.paragraphs, super.key});

  factory ParagraphedSliverList.splitContent(String content, {String splitPattern = '\n\n'}) {
    return ParagraphedSliverList(paragraphs: content.split(splitPattern));
  }

  @override
  Widget build(BuildContext context) {
    return SliverList.separated(
      itemBuilder: (c, i) => BodyText(paragraphs[i]),
      separatorBuilder: (BuildContext context, int index) => const ParagraphSpacer(),
      itemCount: paragraphs.length,
    );
  }
}
