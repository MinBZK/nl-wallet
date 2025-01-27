import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

class HistorySectionHeader extends StatelessWidget {
  final DateTime dateTime;

  const HistorySectionHeader({required this.dateTime, super.key});

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: context.colorScheme.surface,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Divider(),
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
            child: Semantics(
              header: true,
              child: Text.rich(
                DateFormat(DateFormat.YEAR_MONTH, context.l10n.localeName)
                    .format(dateTime)
                    .capitalize
                    .toTextSpan(context),
                maxLines: 1,
                style: context.textTheme.labelSmall,
              ),
            ),
          ),
          const Divider(),
        ],
      ),
    );
  }
}
