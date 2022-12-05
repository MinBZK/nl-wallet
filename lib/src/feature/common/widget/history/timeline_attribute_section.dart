import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

import '../../../../util/extension/string_extension.dart';

class TimelineAttributeSection extends StatelessWidget {
  final DateTime dateTime;

  const TimelineAttributeSection({required this.dateTime, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    AppLocalizations locale = AppLocalizations.of(context);
    return Container(
      color: Theme.of(context).colorScheme.background,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 16.0),
            child: Text(
              DateFormat(DateFormat.YEAR_MONTH, locale.localeName).format(dateTime).capitalize(),
              style: Theme.of(context).textTheme.overline,
            ),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
