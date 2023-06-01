import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import 'data_attribute_row.dart';

class DataAttributeSection extends StatelessWidget {
  final String? sourceCardTitle;
  final List<DataAttribute> attributes;

  const DataAttributeSection({
    required this.sourceCardTitle,
    required this.attributes,
    Key? key,
  }) : super(key: key);

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
    final locale = AppLocalizations.of(context);
    return Text(
      locale.dataAttributeSectionTitle(text),
      style: Theme.of(context).textTheme.bodyMedium,
    );
  }
}
