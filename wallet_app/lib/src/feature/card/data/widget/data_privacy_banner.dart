import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

class DataPrivacyBanner extends StatelessWidget {
  const DataPrivacyBanner({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: context.colorScheme.onBackground,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              context.l10n.cardDataScreenDataPrivacyBanner,
              style: context.textTheme.bodyLarge?.copyWith(
                color: context.colorScheme.background,
              ),
            ),
          ),
          const SizedBox(width: 20),
          SizedBox(
            height: 24,
            width: 24,
            child: Icon(
              Icons.back_hand_outlined,
              color: context.colorScheme.background,
            ),
          ),
        ],
      ),
    );
  }
}
