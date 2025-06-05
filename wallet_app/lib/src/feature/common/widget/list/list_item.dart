import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../button/list_button.dart';
import 'compact_list_item.dart';
import 'horizontal_list_item.dart';
import 'vertical_list_item.dart';

const kListItemIconSize = 24.0;

class ListItem extends StatelessWidget {
  /// The main text displayed in the list item.
  final Widget label;

  /// The secondary text displayed below the label.
  final Widget subtitle;

  /// The leading icon displayed before the text.
  final Widget? icon;

  /// The trailing button displayed at the end of the list item. Only visible when the
  /// [style] is [ListItemStyle.vertical]. For consistency, consider using the
  /// [LinkButton] from 'button/list_button.dart'.
  final Widget? button;

  /// The style of the list item layout (compact, horizontal, vertical).
  final ListItemStyle style;

  /// Specifies if dividers should be placed above or below the item.
  final DividerSide dividerSide;

  const ListItem({
    required this.label,
    required this.subtitle,
    this.icon,
    this.button,
    this.style = ListItemStyle.compact,
    this.dividerSide = DividerSide.none,
    super.key,
  }) : assert(
          button == null || style == ListItemStyle.vertical,
          '[button] can only be rendered in vertical style items',
        );

  const ListItem.compact({
    required this.label,
    required this.subtitle,
    this.icon,
    this.button,
    this.style = ListItemStyle.compact,
    this.dividerSide = DividerSide.none,
    super.key,
  });

  const ListItem.horizontal({
    required this.label,
    required this.subtitle,
    this.icon,
    this.button,
    this.style = ListItemStyle.horizontal,
    this.dividerSide = DividerSide.none,
    super.key,
  });

  const ListItem.vertical({
    required this.label,
    required this.subtitle,
    this.icon,
    this.button,
    this.style = ListItemStyle.vertical,
    this.dividerSide = DividerSide.none,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final styledIcon = _buildIcon(context);
    final item = switch (style) {
      ListItemStyle.compact => CompactListItem(label: label, subtitle: subtitle, icon: styledIcon),
      ListItemStyle.horizontal => HorizontalListItem(label: label, subtitle: subtitle, icon: styledIcon),
      ListItemStyle.vertical => VerticalListItem(label: label, subtitle: subtitle, icon: styledIcon, button: button),
    };
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (dividerSide.top) const Divider(),
        item,
        if (dividerSide.bottom) const Divider(),
      ],
    );
  }

  Widget? _buildIcon(BuildContext context) {
    if (icon == null) return null;
    return SizedBox(
      width: kListItemIconSize,
      height: kListItemIconSize,
      child: IconTheme(
        data: IconThemeData(
          size: kListItemIconSize,
          color: context.theme.iconTheme.color,
        ),
        child: icon!,
      ),
    );
  }
}

enum ListItemStyle { compact, horizontal, vertical }
