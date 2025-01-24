import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';
import '../../../../util/helper/bsn_helper.dart';
import '../../../../util/helper/semantics_helper.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    final prettyValue = attribute.value.prettyPrint(context);
    return MergeSemantics(
      child: Column(
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
      ),
    );
  }
}
