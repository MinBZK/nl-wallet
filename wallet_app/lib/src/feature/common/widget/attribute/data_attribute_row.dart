import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';
import '../../../../util/helper/bsn_helper.dart';
import '../../../../util/helper/semantics_helper.dart';
import '../bullet_list_dot.dart';
import '../list/list_item.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    return ListItem(
      label: _buildLabel(context, attribute.value),
      subtitle: _buildSubtitle(context, attribute.value),
    );
  }

  Widget _buildLabel(BuildContext context, AttributeValue attributeValue) {
    InlineSpan labelSpan = attribute.label.l10nSpan(context);
    if (attributeValue is ArrayValue && attributeValue.value.length > 1) {
      final label = attribute.label.l10nValue(context);
      final length = attributeValue.value.length;
      labelSpan = '$label ($length)'.toTextSpan(context);
    }
    return Text.rich(labelSpan);
  }

  Widget _buildSubtitle(BuildContext context, AttributeValue attributeValue) {
    final prettyValue = attributeValue.prettyPrint(context);

    // Check for non-empty array, this value is not simply formatted and displayed.
    if (attributeValue is ArrayValue && attributeValue.value.isNotEmpty) {
      return ListView.builder(
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        itemBuilder: (c, i) {
          final subtitleRow = _buildSubtitle(context, attributeValue.value[i]);
          return Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              const SizedBox(width: 24, height: 24, child: BulletListDot()),
              Expanded(child: subtitleRow),
            ],
          );
        },
        itemCount: attributeValue.value.length,
      );
    }

    return Text.rich(
      prettyValue.toTextSpan(context),
      semanticsLabel: BsnHelper.isValidBsnFormat(prettyValue) ? SemanticsHelper.splitNumberString(prettyValue) : null,
      style: _resolveSubtitleStyle(context, attribute.value),
    );
  }

  TextStyle? _resolveSubtitleStyle(BuildContext context, AttributeValue attributeValue) {
    switch (attributeValue) {
      case ArrayValue():
        return attributeValue.value.isEmpty ? context.textTheme.bodyLarge : null;
      case NullValue():
        return context.textTheme.bodyLarge;
      case StringValue():
        return attributeValue.value.isEmpty ? context.textTheme.bodyLarge : null;
      case BooleanValue():
      case NumberValue():
      case DateValue():
        return null;
    }
  }
}
