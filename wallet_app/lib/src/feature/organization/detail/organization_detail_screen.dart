import 'package:fimber/fimber.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/placeholder_screen.dart';
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
        title: Text(AppLocalizations.of(context).organizationDetailScreenTitle),
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
        if (state is OrganizationDetailInitial) return const CenteredLoadingIndicator();
        if (state is OrganizationDetailSuccess) return _buildOrganizationDetailLoaded(context, state);
        if (state is OrganizationDetailFailure) return _buildOrganizationDetailFailure(context, state);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildOrganizationDetailLoaded(BuildContext context, OrganizationDetailSuccess state) {
    return Scrollbar(
      thumbVisibility: true,
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
                        child: Text(AppLocalizations.of(context).organizationDetailScreenReportIssueCta),
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
                style: Theme.of(context).textTheme.displayMedium,
              ),
            )
          ],
        ),
        const SizedBox(height: 8),
        Text(
          organization.description,
          style: Theme.of(context).textTheme.bodyLarge,
        ),
      ],
    );
  }

  Widget _buildPolicySection(
    BuildContext context,
    OrganizationDetailSuccess state,
  ) {
    final locale = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          locale.organizationDetailScreenPrivacyHeader,
          textAlign: TextAlign.start,
          style: Theme.of(context).textTheme.bodySmall,
        ),
        const SizedBox(height: 8),
        IconRow(
          icon: const Icon(Icons.policy_outlined),
          text: Text.rich(
            TextSpan(
              text: locale.organizationDetailScreenViewTerms.addSpaceSuffix,
              children: [
                TextSpan(
                  text: locale.organizationDetailScreenPrivacyPolicy,
                  recognizer: TapGestureRecognizer()..onTap = () => PlaceholderScreen.show(context),
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(decoration: TextDecoration.underline),
                )
              ],
            ),
          ),
          padding: const EdgeInsets.symmetric(vertical: 8),
        ),
        if (!state.hasPreviouslyInteractedWithOrganization)
          IconRow(
            icon: Image.asset(
              'assets/images/ic_first_share.png',
              color: Theme.of(context).iconTheme.color,
            ),
            text: Text(locale.organizationDetailScreenFirstInteraction),
            padding: const EdgeInsets.symmetric(vertical: 8),
          ),
      ],
    );
  }

  Widget _buildInfoSection(BuildContext context, Organization organization) {
    final locale = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          locale.organizationDetailScreenInfoHeader,
          textAlign: TextAlign.start,
          style: Theme.of(context).textTheme.bodySmall,
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
            text: Text.rich(TextSpan(
              text: organization.webUrl!,
              recognizer: TapGestureRecognizer()..onTap = () => PlaceholderScreen.show(context),
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(decoration: TextDecoration.underline),
            )),
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
          child: Text(AppLocalizations.of(context).generalRetry),
        ),
      ],
    );
  }
}
