import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'text/body_text.dart';

/// Renders the provided [paragraphs] as [BodyText] with a 8dp separator
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
      itemBuilder: (c, i) => BodyText(paragraphs[i]),
      separatorBuilder: (BuildContext context, int index) => const SizedBox(height: 8),
      itemCount: paragraphs.length,
    );
  }
}
