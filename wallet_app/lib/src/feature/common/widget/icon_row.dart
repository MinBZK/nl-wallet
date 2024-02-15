import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class IconRow extends StatelessWidget {
  final Widget icon;
  final Widget text;
  final EdgeInsets padding;

  const IconRow({
    required this.icon,
    required this.text,
    this.padding = const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: padding,
      child: Row(
        mainAxisSize: MainAxisSize.max,
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          icon,
          const SizedBox(width: 16),
          DefaultTextStyle(
            style: context.textTheme.bodyLarge!,
            child: Expanded(child: text),
          ),
        ],
      ),
    );
  }
}
