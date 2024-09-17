import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/ui_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';
import '../../../../util/helper/bsn_helper.dart';
import '../../../../util/helper/semantics_helper.dart';

class UiAttributeRow extends StatelessWidget {
  final UiAttribute attribute;

  const UiAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    final prettyValue = attribute.value.prettyPrint(context);
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Icon(
          attribute.icon,
          size: 24,
          color: context.colorScheme.onSurfaceVariant,
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
                prettyValue,
                style: context.textTheme.titleMedium,
                semanticsLabel:
                    BsnHelper.isValidBsnFormat(prettyValue) ? SemanticsHelper.splitNumberString(prettyValue) : null,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
