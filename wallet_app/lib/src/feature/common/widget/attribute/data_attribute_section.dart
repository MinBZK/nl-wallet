import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import 'data_attribute_row.dart';

class DataAttributeSection extends StatelessWidget {
  final String sourceCardTitle;
  final List<DataAttribute> attributes;

  const DataAttributeSection({
    required this.sourceCardTitle,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemBuilder: (context, index) {
        if (index == 0) return _buildHeader(context);
        return DataAttributeRow(attribute: attributes[index - 1]);
      },
      separatorBuilder: (index, context) => const SizedBox(height: 16),
      itemCount: attributes.length + 1,
    );
  }

  Widget _buildHeader(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Text(
      locale.dataAttributeSectionTitle(sourceCardTitle),
      style: Theme.of(context).textTheme.bodyMedium,
    );
  }
}
