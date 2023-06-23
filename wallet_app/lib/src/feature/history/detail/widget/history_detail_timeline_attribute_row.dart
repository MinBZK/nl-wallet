import 'package:flutter/material.dart';

import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/history_details_time_formatter.dart';
import '../../../../util/mapper/timeline_attribute_error_status_icon_mapper.dart';
import '../../../../util/mapper/timeline_attribute_status_color_mapper.dart';
import '../../../../util/mapper/timeline_attribute_status_description_text_mapper.dart';
import '../../../../util/mapper/timeline_attribute_status_title_mapper.dart';

const _kStatusIconSize = 24.0;

class HistoryDetailTimelineAttributeRow extends StatelessWidget {
  final TimelineAttribute attribute;

  const HistoryDetailTimelineAttributeRow({
    required this.attribute,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final String titleText = TimelineAttributeStatusTitleTextMapper.map(context, attribute);
    final String descriptionText = TimelineAttributeStatusDescriptionTextMapper.map(context, attribute);

    final IconData? errorStatusIcon = TimelineAttributeErrorStatusIconMapper.map(attribute);
    final Color statusColor = TimelineAttributeStatusColorMapper.map(context, attribute);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              if (errorStatusIcon != null)
                Icon(
                  errorStatusIcon,
                  color: statusColor,
                  size: _kStatusIconSize,
                )
              else
                const SizedBox(
                  width: _kStatusIconSize,
                  height: _kStatusIconSize,
                ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Text(
                      titleText,
                      style: context.textTheme.titleMedium,
                    ),
                    const SizedBox(height: 2),
                    Text(
                      HistoryDetailsTimeFormatter.format(context, attribute.dateTime),
                      style: context.textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 24),
          Text(
            descriptionText,
            style: context.textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }
}
