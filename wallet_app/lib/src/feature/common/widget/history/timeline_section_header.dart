import 'package:flutter/material.dart';
import 'package:intl/intl.dart' hide TextDirection;

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

const timelineSectionHeaderExtent = 37.0;
const _dividerHeight = 1.0;
const _padding = EdgeInsets.symmetric(vertical: 8, horizontal: 16);

class TimelineSectionHeader extends SliverPersistentHeaderDelegate {
  final DateTime dateTime;

  /// The current [textScaler], had to be provided up front to make the widget scale
  /// properly, since we don't have access to the context when resolving [minExtent].
  final TextScaler textScaler;
  final TextStyle? textStyle;

  double? _cachedExtend;

  TimelineSectionHeader({
    required this.dateTime,
    this.textStyle,
    this.textScaler = TextScaler.noScaling,
    Key? key,
  });

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
              maxLines: 1,
              style: textStyle,
            ),
          ),
          const Divider(height: _dividerHeight),
        ],
      ),
    );
  }

  @override
  double get minExtent => _cachedExtend ??= _calculateHeaderHeight();

  @override
  double get maxExtent => minExtent;

  double _calculateHeaderHeight() {
    TextPainter tp = TextPainter(
      text: TextSpan(text: DateFormat(DateFormat.YEAR_MONTH).format(dateTime).capitalize, style: textStyle),
      textScaler: textScaler,
      textDirection: TextDirection.ltr,
    );
    tp.layout();
    return tp.height + _padding.vertical + _dividerHeight;
  }

  @override
  bool shouldRebuild(covariant TimelineSectionHeader oldDelegate) {
    final shouldRebuild =
        oldDelegate.dateTime != dateTime || oldDelegate.textStyle != textStyle || oldDelegate.textScaler != textScaler;
    if (shouldRebuild) {
      /// Invalidate header height cache as it might no longer be valid.
      _cachedExtend = null;
    }
    return shouldRebuild;
  }
}
