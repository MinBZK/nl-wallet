import 'package:flutter/material.dart';

import '../../../../util/extension/string_extension.dart';
import '../list/list_item.dart';

class MissingAttributeRow extends StatelessWidget {
  final String label;

  const MissingAttributeRow({required this.label, super.key});

  @override
  Widget build(BuildContext context) {
    return ListItem(
      label: Text.rich(label.toTextSpan(context)),
      subtitle: const SizedBox.shrink(),
      icon: const Icon(Icons.do_not_disturb_on_outlined),
    );
  }
}
