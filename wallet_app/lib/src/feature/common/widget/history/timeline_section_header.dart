import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

class TimelineSectionHeader extends StatelessWidget {
  final DateTime dateTime;

  const TimelineSectionHeader({required this.dateTime, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      color: context.colorScheme.background,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
            child: Text(
              DateFormat(DateFormat.YEAR_MONTH, context.l10n.localeName).format(dateTime).capitalize,
              style: context.textTheme.labelSmall,
            ),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
