import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

class MissingAttributeRow extends StatelessWidget {
  final String label;

  const MissingAttributeRow({required this.label, super.key});

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const Icon(Icons.do_not_disturb_on_outlined, size: 20),
        const SizedBox(width: 16),
        Text(
          label,
          style: context.textTheme.bodyLarge,
        ),
      ],
    );
  }
}
