import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

const timelineSectionHeaderExtent = 37.0;
const _dividerHeight = 1.0;
const _padding = EdgeInsets.symmetric(vertical: 8, horizontal: 16);

/// This is the height consumed by the [Text] widget, it's used in combination with
/// the provided [textScaleFactor] to calculate the extent.
final _timelineSectionTextHeight = timelineSectionHeaderExtent - _padding.vertical - _dividerHeight;

class TimelineSectionHeader extends SliverPersistentHeaderDelegate {
  final DateTime dateTime;

  /// The current [textScaler], had to be provided up front to make the widget scale
  /// properly, since we don't have access to the context when resolving [minExtent].
  final TextScaler textScaler;

  TimelineSectionHeader({required this.dateTime, this.textScaler = TextScaler.noScaling, Key? key});

  @override
  Widget build(BuildContext context, double shrinkOffset, bool overlapsContent) {
    return Container(
      color: context.colorScheme.background,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: _padding,
            child: Text(
              DateFormat(DateFormat.YEAR_MONTH, context.l10n.localeName).format(dateTime).capitalize,
              style: context.textTheme.labelSmall,
            ),
          ),
          const Divider(height: _dividerHeight),
        ],
      ),
    );
  }

  @override
  double get maxExtent => minExtent;

  @override
  double get minExtent => timelineSectionHeaderExtent + (textScaler.scale(_timelineSectionTextHeight));

  @override
  bool shouldRebuild(covariant TimelineSectionHeader oldDelegate) => oldDelegate.dateTime != dateTime;
}
