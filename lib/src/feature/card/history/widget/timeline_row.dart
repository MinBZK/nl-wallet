import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/timeline_attribute.dart';
import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../../util/formatter/timeline_attribute_text_formatter.dart';
import '../../../../util/mapper/timeline_attribute_type_color_mapper.dart';
import '../../../../util/mapper/timeline_attribute_type_icon_mapper.dart';
import '../../../verification/widget/status_icon.dart';

class TimelineRow extends StatelessWidget {
  final TimelineAttribute attribute;

  const TimelineRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);

    final Color iconColor = TimelineAttributeTypeColorMapper.map(theme, attribute);
    final IconData iconData = TimelineAttributeTypeIconMapper.map(attribute);
    final String timeAgo = TimeAgoFormatter.format(locale, attribute.dateTime);
    final String content = TimelineAttributeTextFormatter.format(locale, attribute);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24.0, horizontal: 16.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.start,
        mainAxisSize: MainAxisSize.max,
        children: [
          SizedBox(
            height: 40,
            width: 40,
            child: StatusIcon(
              color: iconColor,
              icon: iconData,
            ),
          ),
          const SizedBox(width: 16.0),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(timeAgo, style: Theme.of(context).textTheme.bodyText2),
                const SizedBox(height: 2),
                Text(content, style: Theme.of(context).textTheme.bodyText1),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
