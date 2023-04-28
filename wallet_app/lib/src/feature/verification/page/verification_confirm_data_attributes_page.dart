import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../model/verification_flow.dart';
import '../widget/card_attribute_row.dart';

const _kStorageDurationInMonthsFallback = 3;

class VerificationConfirmDataAttributesPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final VerificationFlow flow;

  const VerificationConfirmDataAttributesPage({
    required this.onDecline,
    required this.onAccept,
    required this.flow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        controller: ScrollController(),
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
              title: locale.verificationConfirmDataAttributesCheckAttributesCta,
              onTap: () => PlaceholderScreen.show(context),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverToBoxAdapter(child: _buildConditionsRow(locale, context)),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(
            child: Container(
              alignment: Alignment.center,
              padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
              child: Text(
                locale.verificationConfirmDataAttributesDisclaimer,
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontStyle: FontStyle.italic),
              ),
            ),
          ),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAccept: onAccept,
                acceptText: locale.verificationConfirmDataAttributesPageApproveCta,
                onDecline: onDecline,
                acceptIcon: Icons.arrow_forward,
                declineText: locale.verificationConfirmDataAttributesPageDenyCta,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildConditionsRow(AppLocalizations locale, BuildContext context) {
    // currently defaults to 3 months for mocks with undefined storageDuration
    final storageDurationInMonths = flow.policy.storageDuration?.inMonths ?? _kStorageDurationInMonthsFallback;
    final String subtitle;
    if (flow.policy.dataIsShared) {
      subtitle = locale.verificationConfirmDataAttributesCheckConditionsDataSharedSubtitle(storageDurationInMonths);
    } else {
      subtitle = locale.verificationConfirmDataAttributesCheckConditionsSubtitle(storageDurationInMonths);
    }

    return InfoRow(
      leading: Image.asset('assets/images/ic_policy.png'),
      title: locale.verificationConfirmDataAttributesCheckConditionsCta,
      subtitle: subtitle,
      onTap: () => PlaceholderScreen.show(context),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            AppLocalizations.of(context).verificationConfirmDataAttributesShareWithTitle(flow.organization.name),
            style: Theme.of(context).textTheme.bodySmall,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            AppLocalizations.of(context)
                .verificationConfirmDataAttributesPageShareDataTitle(flow.resolvedAttributes.length),
            style: Theme.of(context).textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: CardAttributeRow(entry: flow.availableAttributes.entries.toList()[index]),
      ),
      childCount: flow.availableAttributes.length,
    );
  }
}

extension _DurationExtension on Duration {
  int get inMonths => inDays ~/ 30;
}