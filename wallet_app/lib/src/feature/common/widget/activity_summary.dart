import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/date_time_extension.dart';
import '../../../util/extension/localized_text_extension.dart';
import '../../../util/extension/string_extension.dart';

class ActivitySummary extends StatelessWidget {
  final List<WalletEvent> events;
  final VoidCallback? onTap;

  const ActivitySummary({
    required this.events,
    this.onTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: '${_resolveTitle(context)}\n${_resolveSubtitle(context)}'.toAttributedString(context),
      onTapHint: context.l10n.generalWCAGSeeAllActivities,
      excludeSemantics: true,
      onTap: onTap,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: Container(
          width: double.infinity,
          decoration: BoxDecoration(
            border: Border.all(color: context.colorScheme.outlineVariant),
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
                      style: context.textTheme.bodySmall?.copyWith(color: context.colorScheme.onSurfaceVariant),
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
    final relevantEvents = switch (mode) {
      ActivityDisplayMode.today => events.where((element) => element.dateTime.isToday),
      ActivityDisplayMode.lastWeek => events.where((element) => element.dateTime.isInLastWeek),
      ActivityDisplayMode.lastMonth => events.where((element) => element.dateTime.isInLastMonth),
    };
    return _resolveSubtitleForEvents(context, relevantEvents.toList());
  }

  String _resolveSubtitleForEvents(BuildContext context, List<WalletEvent> events) {
    final List<String?> subtitleLines = [
      _generateCardsAddedLine(context, events),
      _generatedLoggedInLine(context, events),
      _generateSharedWithLine(context, events),
    ];
    final subtitles = subtitleLines.nonNulls.toList();
    final separator = ' ${context.l10n.activitySummarySeparator} ';
    switch (subtitles.length) {
      case 0:
        return context.l10n.activitySummaryEmpty;
      case 1:
        return '${context.l10n.activitySummaryPrefix} ${subtitles.first}.';
      case 2:
        return '${context.l10n.activitySummaryPrefix} ${subtitles.join(separator)}.';
      case 3:
        return '${context.l10n.activitySummaryPrefix} ${subtitles.first}, ${subtitles.sublist(1).join(separator)}.';
      default:
        throw UnsupportedError('Unsupported subtitles state (length = ${subtitles.length})');
    }
  }

  /// Generate the 'x cards added' line, or return null when no cards were added.
  String? _generateCardsAddedLine(BuildContext context, List<WalletEvent> relevantEvents) {
    final addedCardsCount =
        relevantEvents.whereType<IssuanceEvent>().where((element) => element.status == EventStatus.success).length;
    if (addedCardsCount == 0) return null;
    return context.l10n.activitySummaryCardsAdded(addedCardsCount, addedCardsCount);
  }

  String? _generatedLoggedInLine(BuildContext context, List<WalletEvent> relevantEvents) {
    final loggedInWithOrganizationNames = relevantEvents
        .whereType<DisclosureEvent>()
        .where((element) => element.status == EventStatus.success)
        .where((element) => element.type == DisclosureType.login)
        .map((e) => e.relyingParty.displayName.l10nValue(context))
        .toSet();

    if (loggedInWithOrganizationNames.isEmpty) return null;

    if (loggedInWithOrganizationNames.length == 1) {
      /// User only shared with one organization, generate the 'Shared with X' line from l10n.
      return context.l10n.activitySummaryLoggedInWith(loggedInWithOrganizationNames.first);
    } else if (loggedInWithOrganizationNames.length <= 3) {
      /// User shared with >1 (unique) organizations.
      /// Format the first (length - 1) items as "OrgA, OrgB"
      final commaSeparatedOrganizations =
          loggedInWithOrganizationNames.toList().sublist(0, loggedInWithOrganizationNames.length - 1).join(', ');

      /// Combine the comma separated organisations in a line, where the last organisation is separated by 'and'.
      return context.l10n
          .activitySummaryLoggedInMultiple(loggedInWithOrganizationNames.last, commaSeparatedOrganizations);
    } else {
      /// Shared with >3 organizations, group the organizations as 'Shared with orgX, orgY and 6 others'

      /// Format the first 2 items as "OrgA, OrgB"
      final commaSeparatedOrganizations = loggedInWithOrganizationNames.take(2).join(', ');
      final otherOrganizationsCount = loggedInWithOrganizationNames.length - 2;

      /// Combine the comma separated organisations with a 'x other parties' string.
      return context.l10n.activitySummarySharedWithMultiple(
        context.l10n.activitySummaryLoggedInWithOthers(otherOrganizationsCount),
        commaSeparatedOrganizations,
      );
    }
  }

  /// Generate the 'Shared with orgX, orgY and orgZ' line, or return null when data was shared.
  String? _generateSharedWithLine(BuildContext context, List<WalletEvent> relevantEvents) {
    final sharedWithOrganizationNames = relevantEvents
        .whereType<DisclosureEvent>()
        .where((element) => element.status == EventStatus.success)
        .where((element) => element.type == DisclosureType.regular)
        .map((e) => e.relyingParty.displayName.l10nValue(context))
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
      return context.l10n
          .activitySummarySharedWithMultiple(sharedWithOrganizationNames.last, commaSeparatedOrganizations);
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
    if (events.isEmpty) return ActivityDisplayMode.lastMonth;
    if (events.every((attribute) => attribute.dateTime.isToday)) return ActivityDisplayMode.today;
    if (events.any((element) => element.dateTime.isInLastWeek)) return ActivityDisplayMode.lastWeek;
    if (events.any((element) => element.dateTime.isInLastMonth)) return ActivityDisplayMode.lastMonth;
    return ActivityDisplayMode.lastMonth;
  }
}

enum ActivityDisplayMode { today, lastWeek, lastMonth }
