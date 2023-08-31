import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../verification/model/organization.dart';
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
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          children: [
            OrganizationLogo(image: AssetImage(organization.logoUrl), size: 40),
            const SizedBox(width: 16),
            Expanded(
              child: Text(
                organization.name,
                textAlign: TextAlign.start,
                style: context.textTheme.displayMedium,
              ),
            )
          ],
        ),
        const SizedBox(height: 8),
        Text(
          organization.description,
          style: context.textTheme.bodyLarge,
        ),
      ],
    );
  }

  Widget _buildPolicySection(
    BuildContext context,
    OrganizationDetailSuccess state,
  ) {
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
          text: InkWell(
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
          padding: const EdgeInsets.symmetric(vertical: 8),
        ),
        if (!state.hasPreviouslyInteractedWithOrganization)
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
          text: Text(organization.category),
          padding: const EdgeInsets.symmetric(vertical: 8),
        ),
        if (organization.department != null)
          IconRow(
            icon: const Icon(Icons.meeting_room_outlined),
            text: Text(organization.department!),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
        if (organization.location != null)
          IconRow(
            icon: const Icon(Icons.location_on_outlined),
            text: Text(organization.location!),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
        if (organization.webUrl != null)
          IconRow(
            icon: const Icon(Icons.language_outlined),
            text: InkWell(
              onTap: () => PlaceholderScreen.show(context),
              child: Text.rich(TextSpan(
                text: organization.webUrl!,
                style: context.textTheme.bodyLarge?.copyWith(decoration: TextDecoration.underline),
              )),
            ),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
      ],
    );
  }

  static Future<void> show(BuildContext context, String organizationId, {VoidCallback? onReportIssuePressed}) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (context) {
          return BlocProvider<OrganizationDetailBloc>(
            create: (BuildContext context) => OrganizationDetailBloc(context.read(), context.read())
              ..add(OrganizationLoadTriggered(organizationId: organizationId)),
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
}
