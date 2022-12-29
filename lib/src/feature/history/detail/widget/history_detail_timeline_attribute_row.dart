import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../util/mapper/timeline_attribute_status_description_text_mapper.dart';
import '../../../../util/mapper/timeline_attribute_status_title_mapper.dart';

class HistoryDetailTimelineAttributeRow extends StatelessWidget {
  final TimelineAttribute attribute;

  const HistoryDetailTimelineAttributeRow({
    required this.attribute,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);

    final String titleText = TimelineAttributeStatusTitleTextMapper.map(locale, attribute);
    final String descriptionText = TimelineAttributeStatusDescriptionTextMapper.map(locale, attribute);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24.0, horizontal: 16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            titleText,
            style: Theme.of(context).textTheme.subtitle1,
          ),
          const SizedBox(height: 2),
          Text(
            descriptionText,
            style: Theme.of(context).textTheme.bodyText1,
          ),
        ],
      ),
    );
  }
}
