import 'package:flutter/material.dart';

class TextStylesTab extends StatelessWidget {
  const TextStylesTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
      children: [
        Text('DisplayLarge', style: Theme.of(context).textTheme.displayLarge),
        Text('DisplayMedium', style: Theme.of(context).textTheme.displayMedium),
        Text('DisplaySmall', style: Theme.of(context).textTheme.displaySmall),
        Text('HeadlineMedium', style: Theme.of(context).textTheme.headlineMedium),
        Text('TitleMedium', style: Theme.of(context).textTheme.titleMedium),
        Text('TitleSmall', style: Theme.of(context).textTheme.titleSmall),
        Text('BodyLarge', style: Theme.of(context).textTheme.bodyLarge),
        Text('BodyMedium', style: Theme.of(context).textTheme.bodyMedium),
        Text('LabelLarge', style: Theme.of(context).textTheme.labelLarge),
        Text('BodySmall', style: Theme.of(context).textTheme.bodySmall),
        Text('LabelSmall', style: Theme.of(context).textTheme.labelSmall),
      ],
    );
  }
}
