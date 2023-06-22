import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class TextStylesTab extends StatelessWidget {
  const TextStylesTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
      children: [
        Text('DisplayLarge', style: context.textTheme.displayLarge),
        Text('DisplayMedium', style: context.textTheme.displayMedium),
        Text('DisplaySmall', style: context.textTheme.displaySmall),
        Text('HeadlineMedium', style: context.textTheme.headlineMedium),
        Text('TitleMedium', style: context.textTheme.titleMedium),
        Text('TitleSmall', style: context.textTheme.titleSmall),
        Text('BodyLarge', style: context.textTheme.bodyLarge),
        Text('BodyMedium', style: context.textTheme.bodyMedium),
        Text('LabelLarge', style: context.textTheme.labelLarge),
        Text('BodySmall', style: context.textTheme.bodySmall),
        Text('LabelSmall', style: context.textTheme.labelSmall),
      ],
    );
  }
}
