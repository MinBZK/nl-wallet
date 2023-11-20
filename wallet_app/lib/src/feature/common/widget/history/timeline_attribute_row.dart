import 'package:flutter/material.dart';

import '../../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../../domain/model/timeline/timeline_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../../util/formatter/timeline_attribute_status_formatter.dart';
import '../../../../util/formatter/timeline_attribute_title_formatter.dart';
import '../../../../util/mapper/timeline/interaction_error_status_icon_mapper.dart';
import '../../../../util/mapper/timeline/interaction_status_color_mapper.dart';
import '../../../../util/mapper/timeline/signing_error_status_icon_mapper.dart';
import '../../../../util/mapper/timeline/timeline_attribute_error_status_icon_mapper.dart';
import '../../../../util/mapper/timeline/timeline_attribute_status_color_mapper.dart';
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
    final String titleText =
        TimelineAttributeTitleFormatter.format(context, attribute, showOperationTitle: showOperationTitle);
    final String timeAgoText = TimeAgoFormatter.format(context, attribute.dateTime);

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
                ExcludeSemantics(
                  child: OrganizationLogo(
                    image: attribute.organization.logo,
                    size: _kOrganizationLogoSize,
                  ),
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
                          child: Text(titleText, style: context.textTheme.titleMedium),
                        ),
                      ),
                      _buildTypeRow(context, attribute),
                      Text(timeAgoText, style: context.textTheme.bodySmall),
                    ],
                  ),
                ),
                const SizedBox(width: 16),
                ExcludeSemantics(
                  child: Icon(
                    Icons.chevron_right,
                    color: context.colorScheme.onBackground,
                  ),
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
      final interactionErrorStatusIconMapper = InteractionErrorStatusIconMapper();
      final signingErrorStatusIconMapper = SigningErrorStatusIconMapper();
      final errorStatusIconMapper = TimelineAttributeErrorStatusIconMapper(
        interactionErrorStatusIconMapper,
        signingErrorStatusIconMapper,
      );
      final interactionStatusColorMapper = InteractionStatusColorMapper();
      final statusColorMapper = TimelineAttributeStatusColorMapper(interactionStatusColorMapper);

      final IconData? errorStatusIcon = errorStatusIconMapper.map(attribute);
      final String typeText = TimelineAttributeStatusTextFormatter.map(context, attribute);
      final Color typeTextColor = statusColorMapper.map(context, attribute);

      return Padding(
        padding: const EdgeInsets.only(bottom: 2),
        child: Row(
          children: [
            if (errorStatusIcon != null) ...[
              Icon(errorStatusIcon, color: context.colorScheme.error, size: 16),
              const SizedBox(width: 8)
            ],
            Flexible(
              child: Text(
                typeText,
                style: context.textTheme.bodyLarge?.copyWith(color: typeTextColor),
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
