import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/formatter/country_code_formatter.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_sized_box.dart';
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
      appBar: AppBar(
        title: Text(context.l10n.organizationDetailScreenTitle),
        leading: const BackButton(),
      ),
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
        return switch (state) {
          OrganizationDetailInitial() => const CenteredLoadingIndicator(),
          OrganizationDetailSuccess() => _buildOrganizationDetailLoaded(context, state),
          OrganizationDetailFailure() => _buildOrganizationDetailFailure(context, state),
        };
      },
    );
  }

  Widget _buildOrganizationDetailLoaded(BuildContext context, OrganizationDetailSuccess state) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildHeaderSection(context, state.organization),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 48)),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildPolicySection(context, state),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 48)),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildInfoSection(context, state.organization),
            ),
          ),
          const SliverSizedBox(height: 24),
          onReportIssuePressed == null
              ? const SliverSizedBox()
              : SliverToBoxAdapter(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Divider(height: 1),
                      const SizedBox(height: 12),
                      LinkButton(
                        onPressed: () {
                          Navigator.pop(context);
                          onReportIssuePressed?.call();
                        },
                        customPadding: const EdgeInsets.symmetric(horizontal: 16),
                        child: Text(context.l10n.organizationDetailScreenReportIssueCta),
                      ),
                      const SizedBox(height: 12),
                      const Divider(height: 1),
                    ],
                  ),
                ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context, Organization organization) {
    return MergeSemantics(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              ExcludeSemantics(
                child: OrganizationLogo(image: organization.logo, size: 40),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Text(
                  organization.legalName.l10nValue(context),
                  textAlign: TextAlign.start,
                  style: context.textTheme.displayMedium,
                ),
              )
            ],
          ),
          const SizedBox(height: 8),
          Text(
            organization.description?.l10nValue(context) ?? '',
            style: context.textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }

  Widget _buildPolicySection(BuildContext context, OrganizationDetailSuccess state) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          context.l10n.organizationDetailScreenPrivacyHeader,
          textAlign: TextAlign.start,
          style: context.textTheme.bodySmall,
        ),
        const SizedBox(height: 8),
        IconRow(
          icon: const Icon(Icons.policy_outlined),
          text: Semantics(
            button: true,
            child: InkWell(
              onTap: () => PlaceholderScreen.show(context),
              child: Text.rich(
                TextSpan(
                  text: context.l10n.organizationDetailScreenViewTerms.addSpaceSuffix,
                  children: [
                    TextSpan(
                      text: context.l10n.organizationDetailScreenPrivacyPolicy,
                      style: context.textTheme.bodyLarge?.copyWith(decoration: TextDecoration.underline),
                    )
                  ],
                ),
              ),
            ),
          ),
          padding: const EdgeInsets.symmetric(vertical: 8),
        ),
        if (state.isFirstInteractionWithOrganization)
          IconRow(
            icon: Image.asset(
              WalletAssets.icon_first_share,
              color: context.theme.iconTheme.color,
            ),
            text: Text(context.l10n.organizationDetailScreenFirstInteraction),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
      ],
    );
  }

  Widget _buildInfoSection(BuildContext context, Organization organization) {
    final country = CountryCodeFormatter.format(organization.countryCode);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          context.l10n.organizationDetailScreenInfoHeader,
          textAlign: TextAlign.start,
          style: context.textTheme.bodySmall,
        ),
        const SizedBox(height: 8),
        IconRow(
          icon: const Icon(Icons.apartment),
          text: Text(organization.type?.l10nValue(context) ?? ''),
          padding: const EdgeInsets.symmetric(vertical: 8),
        ),
        if (organization.department != null)
          IconRow(
            icon: const Icon(Icons.meeting_room_outlined),
            text: Text(organization.department!.l10nValue(context)),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
        if (organization.kvk != null)
          IconRow(
            icon: const Icon(Icons.storefront_outlined),
            text: Text(context.l10n.organizationDetailScreenKvk(organization.kvk!)),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
        if (country != null || organization.city != null)
          IconRow(
            icon: const Icon(Icons.location_on_outlined),
            text: Text(_generateLocationLabel(context, country, organization.city)),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
        if (organization.webUrl != null)
          IconRow(
            icon: const Icon(Icons.language_outlined),
            text: Semantics(
              link: true,
              child: InkWell(
                onTap: () => PlaceholderScreen.show(context),
                child: Text.rich(TextSpan(
                  text: organization.webUrl!,
                  style: context.textTheme.bodyLarge?.copyWith(decoration: TextDecoration.underline),
                )),
              ),
            ),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
      ],
    );
  }

  static Future<void> showPreloaded(
    BuildContext context,
    Organization organization,
    bool isFirstInteractionWithOrganization, {
    VoidCallback? onReportIssuePressed,
  }) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (context) {
          return BlocProvider<OrganizationDetailBloc>(
            create: (BuildContext context) => OrganizationDetailBloc.forOrganization(
              context.read(),
              context.read(),
              organization: organization,
              isFirstInteractionWithOrganization: isFirstInteractionWithOrganization,
            ),
            child: OrganizationDetailScreen(onReportIssuePressed: onReportIssuePressed),
          );
        },
      ),
    );
  }

  Widget _buildOrganizationDetailFailure(BuildContext context, OrganizationDetailFailure state) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      mainAxisSize: MainAxisSize.max,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        TextButton(
          onPressed: () {
            context.read<OrganizationDetailBloc>().add(OrganizationLoadTriggered(organizationId: state.organizationId));
          },
          child: Text(context.l10n.generalRetry),
        ),
      ],
    );
  }

  String _generateLocationLabel(BuildContext context, String? country, LocalizedText? city) {
    assert(country != null || city != null, 'At least one of [country, city] needs to be provided');
    final cityLabel = city?.l10nValue(context);
    if (cityLabel == null) return country!;
    if (country == null) return cityLabel;
    return '$cityLabel, $country';
  }
}
