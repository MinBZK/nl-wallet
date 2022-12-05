import 'package:flutter/material.dart';

class DataAttributeRowMissing extends StatelessWidget {
  final String label;

  const DataAttributeRowMissing({required this.label, Key? key}) : super(key: key);

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
          style: Theme.of(context).textTheme.bodyText1,
        ),
      ],
    );
  }
}
