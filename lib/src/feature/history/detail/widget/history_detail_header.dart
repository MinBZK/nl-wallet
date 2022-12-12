import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../verification/model/organization.dart';

class HistoryDetailHeader extends StatelessWidget {
  final Organization organization;
  final DateTime dateTime;

  const HistoryDetailHeader({
    required this.organization,
    required this.dateTime,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        children: [
          SizedBox(
            width: 40,
            height: 40,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: Image.asset(organization.logoUrl),
            ),
          ),
          const SizedBox(width: 16),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                organization.shortName,
                style: Theme.of(context).textTheme.subtitle1,
              ),
              Text(
                TimeAgoFormatter.format(AppLocalizations.of(context), dateTime),
                style: Theme.of(context).textTheme.bodyText1,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
