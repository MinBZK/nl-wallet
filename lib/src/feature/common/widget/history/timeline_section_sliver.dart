import 'package:flutter/material.dart';
import 'package:flutter_sticky_header/flutter_sticky_header.dart';

import '../../../../domain/model/timeline_section.dart';
import 'timeline_attribute_row.dart';
import 'timeline_section_header.dart';

class TimelineSectionSliver extends StatelessWidget {
  final TimelineSection section;
  final bool showOperationTitle;

  const TimelineSectionSliver({required this.section, this.showOperationTitle = true, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverStickyHeader(
      header: TimelineSectionHeader(dateTime: section.dateTime),
      sliver: SliverList(
        delegate: SliverChildBuilderDelegate(
          (context, i) => TimelineAttributeRow(
            attribute: section.attributes[i],
            showOperationTitle: showOperationTitle,
          ),
          childCount: section.attributes.length,
        ),
      ),
    );
  }
}
