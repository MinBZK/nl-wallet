import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/ui_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';

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
          color: context.colorScheme.primary,
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                attribute.label.l10nValue(context),
                style: context.textTheme.bodySmall,
              ),
              Text(
                attribute.value.prettyPrint(context),
                style: context.textTheme.titleMedium,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
