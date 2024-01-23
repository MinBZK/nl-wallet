import 'package:fimber/fimber.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/formatter/country_code_formatter.dart';
import '../../../util/launch_util.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/link_tile_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import 'argument/organization_detail_screen_argument.dart';
import 'bloc/organization_detail_bloc.dart';

class OrganizationDetailScreen extends StatelessWidget {
  static OrganizationDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return OrganizationDetailScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError(
          'Make sure to pass in [organizationDetailScreenArgument] when opening the organizationDetailScreen');
    }
  }

  final VoidCallback? onReportIssuePressed;

  const OrganizationDetailScreen({
    this.onReportIssuePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildBody(),
            ),
            const BottomBackButton(showDivider: true),
          ],
        ),
      ),
    );
  }

  Widget _buildBody() {
    return BlocBuilder<OrganizationDetailBloc, OrganizationDetailState>(
      builder: (context, state) {
        final content = switch (state) {
          OrganizationDetailInitial() => _buildLoadingSliver(),
          OrganizationDetailFailure() => _buildOrganizationDetailFailure(context, state),
          OrganizationDetailSuccess() => _buildOrganizationDetailLoaded(context, state),
        };
        return Scrollbar(
          child: CustomScrollView(
            slivers: [
              SliverWalletAppBar(
                title: _resolveTitle(context),
                actions: [
                  IconButton(
                    onPressed: () => PlaceholderScreen.show(context),
                    icon: const Icon(Icons.help_outline_rounded),
                  ),
                  const CloseButton(),
                ],
              ),
              content,
            ],
          ),
        );
      },
    );
  }

  String _resolveTitle(BuildContext context) {
    final state = context.read<OrganizationDetailBloc>().state;
    if (state is! OrganizationDetailSuccess) return '';
    return context.l10n.organizationDetailScreenTitle(state.organization.displayName.l10nValue(context));
  }

  Widget _buildLoadingSliver() {
    return const SliverFillRemaining(
      hasScrollBody: false,
      child: CenteredLoadingIndicator(),
    );
  }

  Widget _buildOrganizationDetailFailure(BuildContext context, OrganizationDetailFailure state) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(context.l10n.generalBottomBackCta),
        ),
      ),
    );
  }

  Widget _buildOrganizationDetailLoaded(BuildContext context, OrganizationDetailSuccess state) {
    return SliverList.list(
      children: [
        const SizedBox(height: 24),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: _buildHeaderSection(context, state.organization),
        ),
        const SizedBox(height: 24),
        if (state.sharedDataWithOrganizationBefore) ...[
          _buildInteractionRow(context, state),
          const SizedBox(height: 16),
        ],
        _buildInfoSection(context, state.organization),
        const SizedBox(height: 16),
        onReportIssuePressed == null
            ? const SizedBox()
            : LinkTileButton(
                child: Text(context.l10n.organizationDetailScreenReportIssueCta),
                onPressed: () {
                  Navigator.pop(context);
                  onReportIssuePressed?.call();
                },
              ),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context, Organization organization) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ExcludeSemantics(
          child: OrganizationLogo(image: organization.logo, size: 64, fixedRadius: 12),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Text(
            organization.description?.l10nValue(context) ?? '',
            textAlign: TextAlign.start,
            style: context.textTheme.bodyLarge,
          ),
        )
      ],
    );
  }

  Widget _buildInfoSection(BuildContext context, Organization organization) {
    final country = CountryCodeFormatter.format(organization.countryCode);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildCategoryRow(context, organization),
        if (organization.department != null) _buildDepartmentRow(context, organization),
        if (country != null || organization.city != null) _buildLocationRow(context, country, organization),
        if (organization.webUrl != null) _buildWebUrlRow(context, organization.webUrl!),
        if (organization.privacyPolicyUrl != null) _buildPrivacyRow(context, organization.privacyPolicyUrl!),
        if (organization.kvk != null) _buildKvkRow(context, organization),
      ],
    );
  }

  Widget _buildCategoryRow(BuildContext context, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.apartment,
      title: Text(context.l10n.organizationDetailScreenCategoryInfo),
      subtitle: Text(organization.category?.l10nValue(context) ?? ''),
    );
  }

  Widget _buildDepartmentRow(BuildContext context, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.meeting_room_outlined,
      title: Text(context.l10n.organizationDetailScreenDepartmentInfo),
      subtitle: Text(organization.department!.l10nValue(context)),
    );
  }

  Widget _buildLocationRow(BuildContext context, String? country, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.location_on_outlined,
      title: Text(context.l10n.organizationDetailScreenLocationInfo),
      subtitle: Text(_generateLocationLabel(context, country, organization.city)),
    );
  }

  Widget _buildWebUrlRow(BuildContext context, String webUrl) {
    return _buildInfoRow(
      context,
      icon: Icons.language_outlined,
      title: Text(context.l10n.organizationDetailScreenWebsiteInfo),
      subtitle: Semantics(
        link: true,
        child: Text.rich(
          TextSpan(
            text: webUrl,
            style: context.textTheme.bodyLarge!.copyWith(
              fontWeight: FontWeight.w400,
              decoration: TextDecoration.underline,
              color: context.colorScheme.primary,
            ),
            recognizer: TapGestureRecognizer()..onTap = () => launchUrlStringCatching(webUrl),
          ),
        ),
      ),
    );
  }

  Widget _buildPrivacyRow(BuildContext context, String privacyPolicyUrl) {
    return _buildInfoRow(
      context,
      icon: Icons.policy_outlined,
      title: Text(context.l10n.organizationDetailScreenPrivacyInfo),
      subtitle: Semantics(
        link: true,
        child: Text.rich(
          TextSpan(
            text: privacyPolicyUrl,
            style: Theme.of(context).textTheme.bodyLarge!.copyWith(
                  fontWeight: FontWeight.w400,
                  decoration: TextDecoration.underline,
                  color: Theme.of(context).colorScheme.primary,
                ),
            recognizer: TapGestureRecognizer()..onTap = () => launchUrlStringCatching(privacyPolicyUrl),
          ),
        ),
      ),
    );
  }

  Widget _buildKvkRow(BuildContext context, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.storefront_outlined,
      title: Text(context.l10n.organizationDetailScreenKvkInfo),
      subtitle: Text(organization.kvk ?? ''),
    );
  }

  Widget _buildInfoRow(BuildContext context,
      {required IconData icon, required Widget title, required Widget subtitle}) {
    /// Note: not relying on [InfoRow] widget because the styling here is a bit too custom.
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Icon(icon, size: 24),
        ),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.bodySmall!,
                child: title,
              ),
              DefaultTextStyle(
                style: Theme.of(context).textTheme.bodyLarge!.copyWith(fontWeight: FontWeight.w400),
                child: subtitle,
              ),
            ],
          ),
        ),
      ],
    );
  }

  static Future<void> showPreloaded(
    BuildContext context,
    Organization organization,
    bool sharedDataWithOrganizationBefore, {
    VoidCallback? onReportIssuePressed,
  }) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (context) {
          return BlocProvider<OrganizationDetailBloc>(
            create: (BuildContext context) => OrganizationDetailBloc()
              ..add(
                OrganizationProvided(
                  organization: organization,
                  sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore,
                ),
              ),
            child: OrganizationDetailScreen(onReportIssuePressed: onReportIssuePressed),
          );
        },
      ),
    );
  }

  String _generateLocationLabel(BuildContext context, String? country, LocalizedText? city) {
    assert(country != null || city != null, 'At least one of [country, city] needs to be provided');
    final cityLabel = city?.l10nValue(context);
    if (cityLabel == null) return country!;
    if (country == null) return cityLabel;
    return '$cityLabel, $country';
  }

  Widget _buildInteractionRow(BuildContext context, OrganizationDetailSuccess state) {
    String interaction;
    interaction =
      context.l10n.organizationDetailScreenSomeInteractions(state.organization.displayName.l10nValue(context));
    return Column(
      children: [
        const Divider(height: 1),
        const SizedBox(height: 8),
        Row(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Icon(Icons.history_outlined, size: 24),
            ),
            Expanded(
              child: Text(interaction),
            ),
          ],
        ),
        const SizedBox(height: 8),
        const Divider(height: 1),
      ],
    );
  }
}
