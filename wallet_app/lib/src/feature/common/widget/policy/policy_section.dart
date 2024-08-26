import 'package:flutter/material.dart';

import '../../../../domain/model/policy/policy.dart';
import '../../../../navigation/wallet_routes.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../policy/policy_screen_arguments.dart';
import '../button/link_button.dart';
import 'policy_row.dart';

class PolicySection extends StatelessWidget {
  final Policy policy;
  final bool addSignatureRow;

  const PolicySection(this.policy, {this.addSignatureRow = false, super.key});

  @override
  Widget build(BuildContext context) {
    final storageDuration = policy.storageDuration;
    return Column(
      children: [
        if (storageDuration != null)
          PolicyRow(
            icon: Icons.access_time_outlined,
            title: context.l10n.generalPolicyDataRetentionDuration(storageDuration.inDays),
          ),
        PolicyRow(
          icon: Icons.share_outlined,
          title: policy.dataIsShared
              ? context.l10n.generalPolicyDataWillBeShared
              : context.l10n.generalPolicyDataWillNotBeShared,
        ),
        if (addSignatureRow)
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
          child: Padding(
            padding: const EdgeInsets.only(left: 24),
            child: LinkButton(
              onPressed: () => Navigator.pushNamed(
                context,
                WalletRoutes.policyRoute,
                arguments: PolicyScreenArguments(
                  policy: policy,
                  showSignatureRow: addSignatureRow,
                ),
              ),
              text: Text.rich(context.l10n.generalPolicyAllTermsCta.toTextSpan(context)),
            ),
          ),
        ),
      ],
    );
  }
}
