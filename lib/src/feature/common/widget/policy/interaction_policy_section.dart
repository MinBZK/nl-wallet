import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/policy/interaction_policy.dart';
import '../../../../wallet_routes.dart';
import '../link_button.dart';
import 'policy_row.dart';

class InteractionPolicySection extends StatelessWidget {
  final InteractionPolicy interactionPolicy;

  const InteractionPolicySection(this.interactionPolicy, {Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final storageDuration = interactionPolicy.storageDuration;
    return Column(
      children: [
        if (storageDuration != null)
          PolicyRow(
            icon: Icons.access_time_outlined,
            title: locale.generalInteractionPolicyDataRetentionDuration(storageDuration.inDays),
          ),
        PolicyRow(
          icon: Icons.share_outlined,
          title: interactionPolicy.dataIsShared
              ? locale.generalInteractionPolicyDataWillBeShared
              : locale.generalInteractionPolicyDataWillNotBeShared,
        ),
        if (interactionPolicy.dataIsSignature)
          PolicyRow(
            icon: Icons.security_outlined,
            title: locale.generalInteractionPolicyDataIsSignature,
          ),
        if (storageDuration != null && storageDuration.inDays > 0)
          PolicyRow(
            icon: Icons.delete_outline,
            title: interactionPolicy.deletionCanBeRequested
                ? locale.generalInteractionPolicyDataCanBeDeleted
                : locale.generalInteractionPolicyDataCanNotBeDeleted,
          ),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: LinkButton(
            onPressed: () => Navigator.pushNamed(context, WalletRoutes.policyRoute, arguments: interactionPolicy),
            child: Padding(
              padding: const EdgeInsets.only(left: 8.0),
              child: Text(locale.generalInteractionPolicyAllTermsCta),
            ),
          ),
        ),
      ],
    );
  }
}
