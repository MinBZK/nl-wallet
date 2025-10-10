import 'package:flutter/material.dart';

import '../../../../../domain/model/card/status/card_status_metadata.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

/// Small widget to display card status information, such as "Expired" or "Revoked".
class CardStatusInfoLabel extends StatelessWidget {
  final CardStatusMetadata data;

  const CardStatusInfoLabel(
    this.data, {
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final leadingIcon = data.icon;

    return DecoratedBox(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(4),
        color: data.backgroundColor,
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (leadingIcon != null) ...[
              Icon(
                leadingIcon,
                color: data.iconColor,
                size: 16,
              ),
              const SizedBox(width: 8),
            ],

            Text.rich(
              data.text.toTextSpan(context),
              style: context.textTheme.bodySmall?.copyWith(
                color: data.textColor,
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
