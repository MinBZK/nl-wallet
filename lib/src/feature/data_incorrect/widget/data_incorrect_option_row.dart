import 'package:flutter/material.dart';

import '../../common/widget/link_button.dart';

class DataIncorrectOptionRow extends StatelessWidget {
  final String title, description, cta;
  final VoidCallback onTap;

  const DataIncorrectOptionRow({
    required this.title,
    required this.description,
    required this.cta,
    required this.onTap,
    Key? key,
  }) : super(key: key);

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
          child: const Icon(Icons.drive_file_rename_outline),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: Theme.of(context).textTheme.headline3,
                ),
                const SizedBox(height: 8),
                Text(
                  description,
                  style: Theme.of(context).textTheme.bodyText1,
                ),
                LinkButton(
                  onPressed: onTap,
                  customPadding: EdgeInsets.zero,
                  child: Text(cta),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(width: 16),
      ],
    );
  }
}
