import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/timeline_attribute.dart';
import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../../util/formatter/timeline_attribute_title_formatter.dart';
import '../../../../util/mapper/timeline_attribute_type_color_mapper.dart';
import '../../../../util/mapper/timeline_attribute_type_icon_color_mapper.dart';
import '../../../../util/mapper/timeline_attribute_type_icon_mapper.dart';
import '../../../../util/mapper/timeline_attribute_type_mapper.dart';
import '../status_icon.dart';

class TimelineAttributeRow extends StatelessWidget {
  final TimelineAttribute attribute;
  final VoidCallback onPressed;
  final bool showOperationTitle;

  const TimelineAttributeRow({
    required this.attribute,
    required this.onPressed,
    this.showOperationTitle = true,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);

    final String titleText = TimelineAttributeTitleFormatter.format(
      attribute,
      showOperationTitle: showOperationTitle,
    );
    final Color iconColor = TimelineAttributeTypeIconColorMapper.map(theme, attribute);
    final IconData iconData = TimelineAttributeTypeIconMapper.map(attribute);
    final String typeText = TimelineAttributeTypeTextMapper.map(locale, attribute);
    final Color typeTextColor = TimelineAttributeTypeColorMapper.map(theme, attribute);
    final String timeAgoText = TimeAgoFormatter.format(locale, attribute.dateTime);

    return InkWell(
      onTap: onPressed,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 24.0, horizontal: 16.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.start,
              mainAxisSize: MainAxisSize.max,
              children: [
                SizedBox(
                  height: 40,
                  width: 40,
                  child: StatusIcon(color: iconColor, icon: iconData),
                ),
                const SizedBox(width: 16.0),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Visibility(
                        visible: titleText.isNotEmpty,
                        child: Text(titleText, style: Theme.of(context).textTheme.subtitle1),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        typeText,
                        style: Theme.of(context).textTheme.bodyText1?.copyWith(color: typeTextColor),
                      ),
                      const SizedBox(height: 2),
                      Text(timeAgoText, style: Theme.of(context).textTheme.caption),
                    ],
                  ),
                ),
                const SizedBox(width: 16.0),
                Icon(Icons.arrow_forward_ios_outlined, size: 16, color: Theme.of(context).colorScheme.onBackground),
              ],
            ),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
