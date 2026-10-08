import 'package:flutter/material.dart';

import '../../../../domain/model/organization.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../navigation/wallet_routes.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../policy/policy_screen_arguments.dart';
import '../button/link_button.dart';
import 'policy_row.dart';

class PolicySection extends StatelessWidget {
  final Organization relyingParty;
  final Policy policy;
  final bool addSignatureRow;

  const PolicySection({
    required this.relyingParty,
    required this.policy,
    this.addSignatureRow = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        if (addSignatureRow)
          PolicyRow(
            icon: Icons.security_outlined,
            title: context.l10n.generalPolicyDataIsSignature,
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
                  relyingParty: relyingParty,
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
