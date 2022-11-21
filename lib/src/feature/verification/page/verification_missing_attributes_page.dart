import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/data_attribute_row.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../model/verification_request.dart';

class VerificationMissingAttributesPage extends StatelessWidget {
  final VerificationRequest request;
  final VoidCallback onDecline;

  const VerificationMissingAttributesPage({
    required this.request,
    required this.onDecline,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return CustomScrollView(
      restorationId: 'missing_data_attributes_scrollview',
      slivers: <Widget>[
        const SliverToBoxAdapter(child: SizedBox(height: 32)),
        SliverToBoxAdapter(child: _buildHeaderSection(context)),
        const SliverToBoxAdapter(child: SizedBox(height: 20)),
        SliverList(delegate: _getDataAttributesDelegate()),
        const SliverToBoxAdapter(child: SizedBox(height: 20)),
        SliverToBoxAdapter(child: _buildHowToProceedButton(context)),
        const SliverToBoxAdapter(child: Divider(height: 48)),
        SliverFillRemaining(
          hasScrollBody: false,
          fillOverscroll: true,
          child: _buildCloseRequestButton(context),
        ),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            AppLocalizations.of(context).verificationMissingAttributesPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            AppLocalizations.of(context).verificationMissingAttributesPageDescription(request.organization.name),
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: DataAttributeRow(attribute: request.attributes[index]),
      ),
      childCount: request.attributes.length,
    );
  }

  Widget _buildHowToProceedButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => PlaceholderScreen.show(context, 'Wat kan ik doen?'),
        child: Padding(
          padding: const EdgeInsets.only(left: 10),
          child: Text(AppLocalizations.of(context).verificationMissingAttributesPageHowToProceedCta),
        ),
      ),
    );
  }

  Widget _buildCloseRequestButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Align(
        alignment: Alignment.bottomCenter,
        child: ElevatedButton(
          onPressed: onDecline,
          child: Text(AppLocalizations.of(context).verificationMissingAttributesPageCloseCta),
        ),
      ),
    );
  }
}
