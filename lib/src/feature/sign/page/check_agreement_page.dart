import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/confirm_buttons.dart';
import '../../common/widget/document_section.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../model/sign_flow.dart';

const _kContextIllustration = 'assets/non-free/images/sign_illustration_1.png';

class CheckAgreementPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final SignFlow flow;

  const CheckAgreementPage({
    required this.onDecline,
    required this.onAccept,
    required this.flow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverToBoxAdapter(
            child: DocumentSection(
              document: flow.document,
              organization: flow.organization,
              padding: const EdgeInsets.fromLTRB(16, 24, 16, 0),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildTrustProvider(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAccept: onAccept,
                acceptText: AppLocalizations.of(context).checkAgreementPageConfirmCta,
                onDecline: onDecline,
                declineText: AppLocalizations.of(context).checkAgreementPageCancelCta,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Image.asset(
            _kContextIllustration,
            fit: BoxFit.cover,
            width: double.infinity,
          ),
          const SizedBox(height: 32),
          Text(
            AppLocalizations.of(context).checkAgreementPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            AppLocalizations.of(context).checkAgreementPageSubtitle,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => PlaceholderScreen.show(context),
        child: Padding(
          padding: const EdgeInsets.only(left: 8.0),
          child: Text(AppLocalizations.of(context).checkAgreementPageDataIncorrectCta),
        ),
      ),
    );
  }

  Widget _buildTrustProvider(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          Image.asset(flow.trustProvider.logoUrl),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              AppLocalizations.of(context).checkAgreementPageSignProvider(
                flow.organization.shortName,
                flow.trustProvider.name,
              ),
              style: Theme.of(context).textTheme.bodyText1,
            ),
          )
        ],
      ),
    );
  }
}
