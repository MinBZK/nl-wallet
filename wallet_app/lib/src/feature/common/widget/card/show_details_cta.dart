import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

class ShowDetailsCta extends StatelessWidget {
  const ShowDetailsCta({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text.rich(
            context.l10n.showDetailsCta.toTextSpan(context),
            style: context.textTheme.labelLarge,
          ),
          const SizedBox(width: 8),
          Icon(
            Icons.arrow_forward,
            color: context.textTheme.labelLarge?.color,
            size: context.textScaler.scale(16),
          ),
        ],
      ),
    );
  }
}
