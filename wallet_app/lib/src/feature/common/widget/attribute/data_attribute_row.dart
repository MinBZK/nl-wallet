import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';
import '../../../../util/helper/bsn_helper.dart';
import '../../../../util/helper/semantics_helper.dart';
import '../list/list_item.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    final prettyValue = attribute.value.prettyPrint(context);
    return ListItem(
      label: Text.rich(attribute.label.l10nSpan(context)),
      subtitle: Text.rich(
        prettyValue.toTextSpan(context),
        semanticsLabel: BsnHelper.isValidBsnFormat(prettyValue) ? SemanticsHelper.splitNumberString(prettyValue) : null,
        style: _resolveTextStyle(context, attribute.value),
      ),
    );
  }

  TextStyle? _resolveTextStyle(BuildContext context, AttributeValue attributeValue) {
    switch (attributeValue) {
      case ArrayValue():
        return attributeValue.value.isEmpty ? context.textTheme.bodyLarge : null;
      case NullValue():
        return context.textTheme.bodyLarge;
      case StringValue():
      case BooleanValue():
      case NumberValue():
      case DateValue():
        return null;
    }
  }
}
