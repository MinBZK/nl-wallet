import 'package:flutter/material.dart';

import '../../../../domain/model/policy/policy.dart';
import '../../../../navigation/wallet_routes.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../button/link_button.dart';
import 'policy_row.dart';

class PolicySection extends StatelessWidget {
  final Policy policy;

  const PolicySection(this.policy, {super.key});

  @override
  Widget build(BuildContext context) {
    final storageDuration = policy.storageDuration;
    return Column(
      children: [
        if (policy.dataContainsSingleViewProfilePhoto)
          PolicyRow(
            icon: Icons.remove_red_eye_outlined,
            title: context.l10n.generalPolicyDataSingleViewData,
          ),
        if (policy.dataContainsSingleViewProfilePhoto)
          PolicyRow(
            icon: Icons.account_box_outlined,
            title: context.l10n.generalPolicyDataSingleViewProfilePhoto,
          ),
        if (storageDuration != null)
          PolicyRow(
            icon: Icons.access_time_outlined,
            title: context.l10n.generalPolicyDataRetentionDuration(storageDuration.inDays),
          ),
        if (!policy.dataContainsSingleViewProfilePhoto)
          PolicyRow(
            icon: Icons.share_outlined,
            title: policy.dataIsShared
                ? context.l10n.generalPolicyDataWillBeShared
                : context.l10n.generalPolicyDataWillNotBeShared,
          ),
        if (policy.dataIsSignature)
          PolicyRow(
            icon: Icons.security_outlined,
            title: context.l10n.generalPolicyDataIsSignature,
          ),
        if (storageDuration != null && storageDuration.inDays > 0)
          PolicyRow(
            icon: Icons.delete_outline,
            title: policy.deletionCanBeRequested
                ? context.l10n.generalPolicyDataCanBeDeleted
                : context.l10n.generalPolicyDataCanNotBeDeleted,
          ),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: LinkButton(
            onPressed: () => Navigator.pushNamed(context, WalletRoutes.policyRoute, arguments: policy),
            child: Padding(
              padding: const EdgeInsets.only(left: 8),
              child: Text(context.l10n.generalPolicyAllTermsCta),
            ),
          ),
        ),
      ],
    );
  }
}
