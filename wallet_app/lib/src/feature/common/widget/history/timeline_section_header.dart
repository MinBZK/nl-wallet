import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

import '../../../../util/extension/string_extension.dart';

class TimelineSectionHeader extends StatelessWidget {
  final DateTime dateTime;

  const TimelineSectionHeader({required this.dateTime, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Container(
      color: theme.colorScheme.background,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
            child: Text(
              DateFormat(DateFormat.YEAR_MONTH, locale.localeName).format(dateTime).capitalize,
              style: theme.textTheme.labelSmall,
            ),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
