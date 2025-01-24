import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import 'data_attribute_row.dart';

class DataAttributeSection extends StatelessWidget {
  final String? sourceCardTitle;
  final List<DataAttribute> attributes;

  const DataAttributeSection({
    required this.sourceCardTitle,
    required this.attributes,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final showHeader = sourceCardTitle != null;
    final indexExtension = showHeader ? 1 : 0;
    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemBuilder: (context, index) {
        if (index == 0 && sourceCardTitle != null) return _buildHeader(context, sourceCardTitle!);
        return DataAttributeRow(attribute: attributes[index - indexExtension]);
      },
      separatorBuilder: (index, context) => const SizedBox(height: 16),
      itemCount: attributes.length + indexExtension,
    );
  }

  Widget _buildHeader(BuildContext context, String text) {
    return Text(
      context.l10n.dataAttributeSectionTitle(text),
      style: context.textTheme.bodyMedium,
    );
  }
}
