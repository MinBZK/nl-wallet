import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/date_time_extension.dart';
import '../../../util/extension/localized_text_extension.dart';
import '../../../util/extension/string_extension.dart';
import 'default_text_and_focus_style.dart';

class ActivitySummary extends StatefulWidget {
  final List<WalletEvent> events;
  final VoidCallback? onTap;

  const ActivitySummary({
    required this.events,
    this.onTap,
    super.key,
  });

  @override
  State<ActivitySummary> createState() => _ActivitySummaryState();

  ActivityDisplayMode get mode {
    if (events.isEmpty) return ActivityDisplayMode.lastMonth;
    if (events.every((attribute) => attribute.dateTime.isToday)) return ActivityDisplayMode.today;
    if (events.any((element) => element.dateTime.isInLastWeek)) return ActivityDisplayMode.lastWeek;
    if (events.any((element) => element.dateTime.isInLastMonth)) return ActivityDisplayMode.lastMonth;
    return ActivityDisplayMode.lastMonth;
  }
}

class _ActivitySummaryState extends State<ActivitySummary> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(() => setState(() {})));
  }

  @override
  void dispose() {
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final textPressedColor = context.theme.textButtonTheme.style?.foregroundColor?.resolve({WidgetState.pressed});
    return Semantics(
      button: true,
      attributedLabel: '${_resolveTitle(context)}\n${_resolveSubtitle(context)}'.toAttributedString(context),
      onTapHint: context.l10n.generalWCAGSeeAllActivities,
      excludeSemantics: true,
      onTap: widget.onTap,
      child: TextButton.icon(
        onPressed: widget.onTap,
        icon: const Icon(Icons.arrow_forward),
        iconAlignment: IconAlignment.end,
        statesController: _statesController,
        style: context.theme.iconButtonTheme.style?.copyWith(
          foregroundColor: WidgetStateProperty.resolveWith(
            // Only override the color when the button is not pressed or focused
            (states) => states.isPressedOrFocused ? null : context.colorScheme.onSurface,
          ),
          side: WidgetStateProperty.resolveWith(_resolveBorderSide),
        ),
        label: Padding(
          padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 8),
          child: Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    DefaultTextAndFocusStyle(
                      statesController: _statesController,
                      textStyle: context.textTheme.bodySmall,
                      pressedOrFocusedColor: textPressedColor,
                      child: Text(
                        _resolveTitle(context),
                      ),
                    ),
                    const SizedBox(height: 8),
                    DefaultTextAndFocusStyle(
                      statesController: _statesController,
                      textStyle: context.textTheme.bodyLarge,
                      pressedOrFocusedColor: textPressedColor,
                      child: Text(
                        _resolveSubtitle(context),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  BorderSide? _resolveBorderSide(Set<WidgetState> states) {
    // Override all non-focused states to always display a border
    return !states.isFocused
        ? BaseWalletTheme.buttonBorderSideFocused.copyWith(
            color: context.colorScheme.outlineVariant,
            strokeAlign: BorderSide.strokeAlignOutside,
            width: 1,
          )
        : null;
  }

  String _resolveTitle(BuildContext context) {
    switch (widget.mode) {
      case ActivityDisplayMode.today:
        return context.l10n.activitySummaryToday;
      case ActivityDisplayMode.lastWeek:
        return context.l10n.activitySummaryLastWeek;
      case ActivityDisplayMode.lastMonth:
        return context.l10n.activitySummaryLastMonth;
    }
  }

  String _resolveSubtitle(BuildContext context) {
    final relevantEvents = switch (widget.mode) {
      ActivityDisplayMode.today => widget.events.where((element) => element.dateTime.isToday),
      ActivityDisplayMode.lastWeek => widget.events.where((element) => element.dateTime.isInLastWeek),
      ActivityDisplayMode.lastMonth => widget.events.where((element) => element.dateTime.isInLastMonth),
    };
    return _resolveSubtitleForEvents(context, relevantEvents.toList());
  }

  String _resolveSubtitleForEvents(BuildContext context, List<WalletEvent> events) {
    final List<String?> subtitleLines = [
      _generateCardsAddedLine(context, events),
      _generateCardsUpdatedLine(context, events),
      _generatedLoggedInLine(context, events),
      _generateSharedWithLine(context, events),
    ];
    final subtitles = subtitleLines.nonNulls.toList();
    final separator = ' ${context.l10n.activitySummarySeparator} ';

    if (subtitles.isEmpty) return context.l10n.activitySummaryEmpty;
    if (subtitles.length == 1) return '${context.l10n.activitySummaryPrefix} ${subtitles.first}.';
    if (subtitles.length == 2) return '${context.l10n.activitySummaryPrefix} ${subtitles.join(separator)}.';
    return '${context.l10n.activitySummaryPrefix} ${subtitles.sublist(0, subtitles.length - 1).join(', ')}$separator${subtitles.last}.';
  }

  /// Generate the 'x cards added' line, or return null when no cards were added.
  String? _generateCardsAddedLine(BuildContext context, List<WalletEvent> relevantEvents) {
    final addedCardsCount = relevantEvents
        .whereType<IssuanceEvent>()
        .where((it) => it.status == EventStatus.success)
        .where((it) => !it.renewed)
        .length;
    if (addedCardsCount == 0) return null;
    return context.l10n.activitySummaryCardsAdded(addedCardsCount, addedCardsCount);
  }

  /// Generate the 'x cards replaced' line, or return null when no cards were updated.
  String? _generateCardsUpdatedLine(BuildContext context, List<WalletEvent> relevantEvents) {
    final updatedCardsCount = relevantEvents
        .whereType<IssuanceEvent>()
        .where((it) => it.status == EventStatus.success)
        .where((it) => it.renewed)
        .length;
    if (updatedCardsCount == 0) return null;
    return context.l10n.activitySummaryCardsUpdated(updatedCardsCount, updatedCardsCount);
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
}

enum ActivityDisplayMode { today, lastWeek, lastMonth }
