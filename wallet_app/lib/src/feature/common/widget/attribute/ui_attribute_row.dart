import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';
import '../../../../util/helper/bsn_helper.dart';
import '../../../../util/helper/semantics_helper.dart';

class UiAttributeRow extends StatelessWidget {
  final UiAttribute attribute;

  const UiAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    // Render a simplified version when no icon is provided.
    if (attribute.icon == null) return _buildDataColumn(context);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Icon(
          attribute.icon,
          size: 24,
          color: context.colorScheme.onSurfaceVariant,
        ),
        const SizedBox(width: 16),
        Expanded(child: _buildDataColumn(context)),
      ],
    );
  }

  /// Column that renders the label and the value
  Widget _buildDataColumn(BuildContext context) {
    final prettyValue = attribute.value.prettyPrint(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text.rich(
          attribute.label.l10nSpan(context),
          style: context.textTheme.bodySmall,
        ),
        Text(
          prettyValue,
          style: context.textTheme.titleMedium,
          semanticsLabel:
              BsnHelper.isValidBsnFormat(prettyValue) ? SemanticsHelper.splitNumberString(prettyValue) : null,
        ),
      ],
    );
  }
}
