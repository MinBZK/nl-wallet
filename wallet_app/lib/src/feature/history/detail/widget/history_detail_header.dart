import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../util/formatter/time_ago_formatter.dart';
import '../../../common/widget/organization/organization_logo.dart';
import '../../../verification/model/organization.dart';

const _kOrganizationLogoSize = 40.0;

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
          OrganizationLogo(
            image: AssetImage(organization.logoUrl),
            size: _kOrganizationLogoSize,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  organization.shortName,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                Text(
                  TimeAgoFormatter.format(AppLocalizations.of(context), dateTime),
                  style: Theme.of(context).textTheme.bodyLarge,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
