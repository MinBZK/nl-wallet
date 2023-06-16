import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../../util/formatter/timeline_attribute_title_formatter.dart';
import '../../../../util/mapper/timeline_attribute_error_status_icon_mapper.dart';
import '../../../../util/mapper/timeline_attribute_status_color_mapper.dart';
import '../../../../util/mapper/timeline_attribute_status_mapper.dart';
import '../organization/organization_logo.dart';

const _kOrganizationLogoSize = 40.0;

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

    final String titleText = TimelineAttributeTitleFormatter.format(attribute, showOperationTitle: showOperationTitle);
    final String timeAgoText = TimeAgoFormatter.format(locale, attribute.dateTime);

    return InkWell(
      onTap: onPressed,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.start,
              mainAxisSize: MainAxisSize.max,
              children: [
                OrganizationLogo(
                  image: AssetImage(attribute.organization.logoUrl),
                  size: _kOrganizationLogoSize,
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Visibility(
                        visible: titleText.isNotEmpty,
                        child: Padding(
                          padding: const EdgeInsets.only(bottom: 2),
                          child: Text(titleText, style: theme.textTheme.titleMedium),
                        ),
                      ),
                      _buildTypeRow(context, attribute),
                      Text(timeAgoText, style: theme.textTheme.bodySmall),
                    ],
                  ),
                ),
                const SizedBox(width: 16),
                Icon(
                  Icons.chevron_right,
                  color: theme.colorScheme.onBackground,
                ),
              ],
            ),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }

  /// Currently we do not show the type row for [InteractionTimelineAttribute] with status [InteractionStatus.success].
  /// This is a design choice build on the thought that successful interactions are the main goal of the wallet.
  Widget _buildTypeRow(BuildContext context, TimelineAttribute attribute) {
    final bool hideTypeRow = attribute is InteractionTimelineAttribute && attribute.status == InteractionStatus.success;
    if (!hideTypeRow) {
      final locale = AppLocalizations.of(context);
      final theme = Theme.of(context);

      final IconData? errorStatusIcon = TimelineAttributeErrorStatusIconMapper.map(attribute);
      final String typeText = TimelineAttributeStatusTextMapper.map(locale, attribute);
      final Color typeTextColor = TimelineAttributeStatusColorMapper.map(theme, attribute);

      return Padding(
        padding: const EdgeInsets.only(bottom: 2),
        child: Row(
          children: [
            if (errorStatusIcon != null) ...[
              Icon(errorStatusIcon, color: theme.colorScheme.error, size: 16),
              const SizedBox(width: 8)
            ],
            Flexible(
              child: Text(
                typeText,
                style: theme.textTheme.bodyLarge?.copyWith(color: typeTextColor),
              ),
            ),
          ],
        ),
      );
    } else {
      return const SizedBox();
    }
  }
}
