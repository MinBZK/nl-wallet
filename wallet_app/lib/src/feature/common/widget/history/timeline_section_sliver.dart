import 'package:flutter/material.dart';

import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_section.dart';
import 'timeline_attribute_row.dart';
import 'timeline_section_header.dart';

class TimelineSectionSliver extends StatelessWidget {
  final TimelineSection section;
  final Function(String timelineAttributeId) onRowPressed;
  final bool showOperationTitle;

  const TimelineSectionSliver({
    required this.section,
    required this.onRowPressed,
    this.showOperationTitle = true,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverMainAxisGroup(
      slivers: [
        SliverPersistentHeader(
          pinned: true,
          delegate: TimelineSectionHeader(
            dateTime: section.dateTime,
            textScaleFactor: MediaQuery.of(context).textScaleFactor,
          ),
        ),
        SliverList(
          delegate: SliverChildBuilderDelegate(
            (context, i) {
              final TimelineAttribute attribute = section.attributes[i];
              return TimelineAttributeRow(
                attribute: attribute,
                onPressed: () => onRowPressed(attribute.id),
                showOperationTitle: showOperationTitle,
              );
            },
            childCount: section.attributes.length,
          ),
        )
      ],
    );
  }
}
