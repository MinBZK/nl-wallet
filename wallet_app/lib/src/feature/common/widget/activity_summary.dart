import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/date_time_extension.dart';

class ActivitySummary extends StatelessWidget {
  final List<TimelineAttribute> attributes;
  final VoidCallback? onTap;

  const ActivitySummary({
    required this.attributes,
    this.onTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(14),
      child: Container(
        width: double.infinity,
        decoration: BoxDecoration(
          border: Border.all(color: (context.colorScheme.outlineVariant)),
          borderRadius: BorderRadius.circular(14),
        ),
        padding: const EdgeInsets.all(24),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    _resolveTitle(context),
                    style: context.textTheme.bodySmall?.copyWith(color: context.colorScheme.onSurface),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    _resolveSubtitle(context),
                    style: context.textTheme.bodyLarge,
                  ),
                ],
              ),
            ),
            const SizedBox(width: 8),
            Icon(
              Icons.arrow_forward,
              color: context.colorScheme.primary,
            ),
          ],
        ),
      ),
    );
  }

  String _resolveTitle(BuildContext context) {
    switch (mode) {
      case ActivityDisplayMode.today:
        return context.l10n.activitySummaryToday;
      case ActivityDisplayMode.lastWeek:
        return context.l10n.activitySummaryLastWeek;
      case ActivityDisplayMode.lastMonth:
        return context.l10n.activitySummaryLastMonth;
    }
  }

  String _resolveSubtitle(BuildContext context) {
    final relevantAttributes = switch (mode) {
      ActivityDisplayMode.today => attributes.where((element) => element.dateTime.isToday),
      ActivityDisplayMode.lastWeek => attributes.where((element) => element.dateTime.isInLastWeek),
      ActivityDisplayMode.lastMonth => attributes.where((element) => element.dateTime.isInLastMonth),
    };
    return _resolveSubtitleForAttributes(context, relevantAttributes.toList());
  }

  String _resolveSubtitleForAttributes(BuildContext context, List<TimelineAttribute> relevantAttributes) {
    List<String?> subtitleLines = [
      _generateCardsAddedLine(context, relevantAttributes),
      _generateSharedWithLine(context, relevantAttributes),
    ];
    if (subtitleLines.nonNulls.isEmpty) return context.l10n.activitySummaryEmpty;
    return subtitleLines.nonNulls.join(' ');
  }

  /// Generate the 'x cards added' line, or return null when no cards were added.
  String? _generateCardsAddedLine(BuildContext context, List<TimelineAttribute> relevantAttributes) {
    final addedCardsCount = relevantAttributes
        .whereType<OperationTimelineAttribute>()
        .where((element) => element.status == OperationStatus.issued)
        .length;
    if (addedCardsCount == 0) return null;
    return context.l10n.activitySummaryCardsAdded(addedCardsCount, addedCardsCount);
  }

  /// Generate the 'Shared with orgX, orgY and orgZ' line, or return null when data was shared.
  String? _generateSharedWithLine(BuildContext context, List<TimelineAttribute> relevantAttributes) {
    final sharedWithOrganizationNames = relevantAttributes
        .whereType<InteractionTimelineAttribute>()
        .where((element) => element.status == InteractionStatus.success)
        .map((e) => e.organization.displayName.l10nValue(context))
        .toSet();

    if (sharedWithOrganizationNames.isEmpty) return null;

    if (sharedWithOrganizationNames.length == 1) {
      /// User only shared with one organization, generate the 'Shared with X' line from l10n.
      return context.l10n.activitySummarySharedWith(sharedWithOrganizationNames.first);
    } else if (sharedWithOrganizationNames.length <= 3) {
      /// User shared with >1 (unique) organizations.
      /// Format the first (length - 1) items as "OrgA, OrgB"
      final commaSeparatedOrganizations =
          sharedWithOrganizationNames.toList().sublist(0, sharedWithOrganizationNames.length - 1).join(', ');

      /// Combine the comma separated organisations in a line, where the last organisation is separated by 'and'.
      return context.l10n.activitySummarySharedWithMultiple(
        sharedWithOrganizationNames.last,
        commaSeparatedOrganizations,
      );
    } else {
      /// Shared with >3 organizations, group the organizations as 'Shared with orgX, orgY and 6 others'

      /// Format the first 2 items as "OrgA, OrgB"
      final commaSeparatedOrganizations = sharedWithOrganizationNames.take(2).join(', ');
      final otherOrganizationsCount = sharedWithOrganizationNames.length - 2;

      /// Combine the comma separated organisations with a 'x other parties' string.
      return context.l10n.activitySummarySharedWithMultiple(
        context.l10n.activitySummarySharedWithOthers(otherOrganizationsCount),
        commaSeparatedOrganizations,
      );
    }
  }

  ActivityDisplayMode get mode {
    if (attributes.every((attribute) => attribute.dateTime.isToday)) return ActivityDisplayMode.today;
    if (attributes.any((element) => element.dateTime.isInLastWeek)) return ActivityDisplayMode.lastWeek;
    if (attributes.any((element) => element.dateTime.isInLastMonth)) return ActivityDisplayMode.lastMonth;
    return ActivityDisplayMode.lastMonth;
  }
}

enum ActivityDisplayMode { today, lastWeek, lastMonth }
