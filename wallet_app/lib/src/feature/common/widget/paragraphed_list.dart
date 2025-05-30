import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'spacer/paragraph_spacer.dart';
import 'text/body_text.dart';

/// Renders the provided [paragraphs] as [BodyText] with a 8dp separator
/// Check out the [ParagraphedSliverList] for use with slivers.
class ParagraphedList extends StatelessWidget {
  final List<String> paragraphs;

  const ParagraphedList({required this.paragraphs, super.key});

  factory ParagraphedList.splitContent(String content, {String splitPattern = '\n\n'}) {
    return ParagraphedList(paragraphs: content.split(splitPattern));
  }

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemBuilder: (c, i) => BodyText(paragraphs[i]),
      separatorBuilder: (BuildContext context, int index) => const ParagraphSpacer(),
      itemCount: paragraphs.length,
    );
  }
}
