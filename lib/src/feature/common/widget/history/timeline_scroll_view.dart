import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_sticky_header/flutter_sticky_header.dart';

import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/timeline_attribute.dart';
import '../../../../util/extension/date_time_extension.dart';
import '../text_icon_button.dart';
import 'timeline_attribute_row.dart';
import 'timeline_attribute_section.dart';
import 'timeline_card_header.dart';

class TimelineScrollView extends StatelessWidget {
  final List<TimelineAttribute> attributes;
  final CardFront? cardFront;

  const TimelineScrollView({required this.attributes, this.cardFront, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final List<Widget> slivers = [];

    // Card front
    final front = cardFront;
    final isMultiCardHistoryContext = front == null;
    if (!isMultiCardHistoryContext) {
      slivers.add(SliverToBoxAdapter(child: TimelineCardHeader(cardFront: front)));
    }

    // Timeline
    final Map<DateTime, List<TimelineAttribute>> monthYearMap = _monthYearAttributeMap(attributes);
    monthYearMap.forEach((dateTime, values) {
      slivers.add(
        SliverStickyHeader(
          header: TimelineAttributeSection(dateTime: dateTime),
          sliver: SliverList(
            delegate: SliverChildBuilderDelegate(
              (context, i) => TimelineAttributeRow(
                attribute: values[i],
                showOperationTitle: isMultiCardHistoryContext,
              ),
              childCount: values.length,
            ),
          ),
        ),
      );
    });

    // Close button
    slivers.add(
      SliverFillRemaining(
        hasScrollBody: false,
        fillOverscroll: true,
        child: _buildBackButton(context),
      ),
    );

    return CustomScrollView(slivers: slivers);
  }

  Widget _buildBackButton(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: SizedBox(
        height: 72,
        width: double.infinity,
        child: TextIconButton(
          onPressed: () => Navigator.pop(context),
          iconPosition: IconPosition.start,
          icon: Icons.arrow_back,
          child: Text(AppLocalizations.of(context).timelineScrollViewBackCta),
        ),
      ),
    );
  }

  Map<DateTime, List<TimelineAttribute>> _monthYearAttributeMap(List<TimelineAttribute> attributes) {
    Map<DateTime, List<TimelineAttribute>> map = {};

    for (TimelineAttribute attribute in attributes) {
      final DateTime yearMonthKey = attribute.dateTime.yearMonthOnly();

      List<TimelineAttribute>? mapEntry = map[yearMonthKey];
      if (mapEntry != null) {
        mapEntry.add(attribute);
      } else {
        map[yearMonthKey] = [attribute];
      }
    }
    return map;
  }
}
