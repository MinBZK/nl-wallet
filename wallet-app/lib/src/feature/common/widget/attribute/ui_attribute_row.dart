import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/ui_attribute.dart';

class UiAttributeRow extends StatelessWidget {
  final UiAttribute attribute;

  const UiAttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Icon(
          attribute.icon,
          size: 24,
          color: Theme.of(context).colorScheme.primary,
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                attribute.label,
                style: Theme.of(context).textTheme.bodySmall,
              ),
              Text(
                attribute.value,
                style: Theme.of(context).textTheme.titleMedium,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
