import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class IconRow extends StatelessWidget {
  final Widget icon;
  final Widget text;
  final EdgeInsets padding;
  final CrossAxisAlignment? crossAxisAlignment;

  const IconRow({
    required this.icon,
    required this.text,
    this.padding = const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
    this.crossAxisAlignment,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: padding,
      child: Row(
        mainAxisSize: MainAxisSize.max,
        crossAxisAlignment: crossAxisAlignment ?? CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          icon,
          const SizedBox(width: 4),
          DefaultTextStyle(
            style: context.textTheme.bodyLarge!,
            child: Expanded(child: text),
          ),
        ],
      ),
    );
  }
}
