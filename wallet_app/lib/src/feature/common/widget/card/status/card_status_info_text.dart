import 'package:flutter/material.dart';

import '../../../../../domain/model/card/status/card_status_metadata.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../text/body_text.dart';

/// Explanatory (multiline) text to display card status information, such as "Expired" or "Revoked",
/// including optional info like expiration date or not valid before date.
class CardStatusInfoText extends StatelessWidget {
  final CardStatusMetadata data;

  const CardStatusInfoText(
    this.data, {
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final leadingIcon = data.icon;
    final bodyText = BodyText(
      data.text,
      style: context.textTheme.bodyLarge?.copyWith(
        color: data.textColor,
      ),
    );

    if (leadingIcon == null) return bodyText;

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 3),
          child: Icon(
            leadingIcon,
            color: data.iconColor,
          ),
        ),
        const SizedBox(width: 8),
        Flexible(
          child: bodyText,
        ),
      ],
    );
  }
}
