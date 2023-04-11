import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/policy/policy.dart';
import '../../../../wallet_routes.dart';
import '../link_button.dart';
import 'policy_row.dart';

class PolicySection extends StatelessWidget {
  final Policy policy;

  const PolicySection(this.policy, {Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final storageDuration = policy.storageDuration;
    return Column(
      children: [
        if (policy.dataContainsSingleViewProfilePhoto)
          PolicyRow(
            icon: Icons.remove_red_eye_outlined,
            title: locale.generalPolicyDataSingleViewData,
          ),
        if (policy.dataContainsSingleViewProfilePhoto)
          PolicyRow(
            icon: Icons.account_box_outlined,
            title: locale.generalPolicyDataSingleViewProfilePhoto,
          ),
        if (storageDuration != null)
          PolicyRow(
            icon: Icons.access_time_outlined,
            title: locale.generalPolicyDataRetentionDuration(storageDuration.inDays),
          ),
        if (!policy.dataContainsSingleViewProfilePhoto)
          PolicyRow(
            icon: Icons.share_outlined,
            title: policy.dataIsShared ? locale.generalPolicyDataWillBeShared : locale.generalPolicyDataWillNotBeShared,
          ),
        if (policy.dataIsSignature)
          PolicyRow(
            icon: Icons.security_outlined,
            title: locale.generalPolicyDataIsSignature,
          ),
        if (storageDuration != null && storageDuration.inDays > 0)
          PolicyRow(
            icon: Icons.delete_outline,
            title: policy.deletionCanBeRequested
                ? locale.generalPolicyDataCanBeDeleted
                : locale.generalPolicyDataCanNotBeDeleted,
          ),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: LinkButton(
            onPressed: () => Navigator.pushNamed(context, WalletRoutes.policyRoute, arguments: policy),
            child: Padding(
              padding: const EdgeInsets.only(left: 8.0),
              child: Text(locale.generalPolicyAllTermsCta),
            ),
          ),
        ),
      ],
    );
  }
}
