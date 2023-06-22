import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/duration_extension.dart';
import '../../check_attributes/check_attributes_screen.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../policy/policy_screen.dart';
import '../model/verification_flow.dart';
import '../widget/card_attribute_row.dart';

const _kStorageDurationInMonthsFallback = 3;

class VerificationConfirmDataAttributesPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final VoidCallback? onReportIssuePressed;
  final VerificationFlow flow;

  const VerificationConfirmDataAttributesPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    this.onReportIssuePressed,
    required this.flow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        restorationId: 'confirm_data_attributes_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverSizedBox(height: 24),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverToBoxAdapter(
            child: InfoRow(
              icon: Icons.remove_red_eye_outlined,
              title: context.l10n.verificationConfirmDataAttributesCheckAttributesCta,
              onTap: () => CheckAttributesScreen.show(
                context,
                flow.availableAttributes,
                onDataIncorrectPressed: () {
                  Navigator.pop(context);
                  onReportIssuePressed?.call();
                },
              ),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverToBoxAdapter(child: _buildConditionsRow(context)),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBottomSection(context),
          ),
        ],
      ),
    );
  }

  Widget _buildConditionsRow(BuildContext context) {
    // currently defaults to 3 months for mocks with undefined storageDuration
    final storageDurationInMonths = flow.policy.storageDuration?.inMonths ?? _kStorageDurationInMonthsFallback;
    final String subtitle;
    if (flow.policy.dataIsShared) {
      subtitle =
          context.l10n.verificationConfirmDataAttributesCheckConditionsDataSharedSubtitle(storageDurationInMonths);
    } else {
      subtitle = context.l10n.verificationConfirmDataAttributesCheckConditionsSubtitle(storageDurationInMonths);
    }

    return InfoRow(
      leading: Image.asset('assets/images/ic_policy.png'),
      title: context.l10n.verificationConfirmDataAttributesCheckConditionsCta,
      subtitle: subtitle,
      onTap: () => PolicyScreen.show(
        context,
        flow.policy,
        onReportIssuePressed: onReportIssuePressed,
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            context.l10n.verificationConfirmDataAttributesShareWithTitle(flow.organization.name),
            style: context.textTheme.bodySmall,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.verificationConfirmDataAttributesPageShareDataTitle(flow.resolvedAttributes.length),
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    final entries = flow.availableAttributes.entries.toList();
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: CardAttributeRow(entry: entries[index]),
      ),
      childCount: flow.availableAttributes.length,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      children: [
        const SizedBox(height: 24),
        Container(
          alignment: Alignment.center,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            context.l10n.verificationConfirmDataAttributesDisclaimer,
            style: context.textTheme.bodyMedium?.copyWith(fontStyle: FontStyle.italic),
          ),
        ),
        Container(
          alignment: Alignment.bottomCenter,
          child: ConfirmButtons(
            onAcceptPressed: onAcceptPressed,
            acceptText: context.l10n.verificationConfirmDataAttributesPageApproveCta,
            onDeclinePressed: onDeclinePressed,
            acceptIcon: Icons.arrow_forward,
            declineText: context.l10n.verificationConfirmDataAttributesPageDenyCta,
          ),
        ),
      ],
    );
  }
}
