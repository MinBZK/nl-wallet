import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/link_button.dart';

class DataIncorrectOptionRow extends StatelessWidget {
  final String title, description, cta;
  final IconData icon;
  final VoidCallback onTap;

  const DataIncorrectOptionRow({
    required this.title,
    required this.description,
    required this.cta,
    required this.icon,
    required this.onTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          height: 56,
          width: 56,
          margin: const EdgeInsets.only(top: 8),
          alignment: Alignment.center,
          child: Icon(icon),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 24),
            child: MergeSemantics(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: context.textTheme.displaySmall,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    description,
                    style: context.textTheme.bodyLarge,
                  ),
                  LinkButton(
                    onPressed: onTap,
                    text: Text(cta),
                  ),
                ],
              ),
            ),
          ),
        ),
        const SizedBox(width: 16),
      ],
    );
  }
}
