import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/disclosure/disclose_card_request.dart';
import '../../domain/model/organization.dart';
import '../../domain/model/policy/organization_policy.dart';
import '../../domain/model/policy/policy.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/mapper/context_mapper.dart';
import '../../wallet_constants.dart';
import '../check_attributes/check_attributes_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/card/shared_attributes_card.dart';
import '../common/widget/organization/organization_logo.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../info/info_screen.dart';
import '../organization/detail/organization_detail_screen.dart';
import '../policy/policy_screen.dart';
import 'argument/login_detail_screen_argument.dart';

class LoginDetailScreen extends StatelessWidget {
  final Organization organization;
  final Policy policy;
  final List<DiscloseCardRequest> cardRequests;
  final bool sharedDataWithOrganizationBefore;
  final VoidCallback? onReportIssuePressed;

  static LoginDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args! as LoginDetailScreenArgument;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [LoginDetailScreenArgument] when opening the LoginDetailScreen');
    }
  }

  const LoginDetailScreen({
    required this.organization,
    required this.policy,
    required this.cardRequests,
    required this.sharedDataWithOrganizationBefore,
    this.onReportIssuePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.loginDetailScreenTitle(organization.displayName.l10nValue(context))),
        actions: const [HelpIconButton()],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildBody(context),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(
            child: Padding(
              padding: kDefaultTitlePadding,
              child: TitleText(context.l10n.loginDetailScreenTitle(organization.displayName.l10nValue(context))),
            ),
          ),
          const SliverSizedBox(height: 12),
          const SliverDivider(),
          _buildOrganizationSection(context),
          const SliverDivider(),
          _buildAttributesSection(context),
          const SliverDivider(),
          _buildAgreementSection(context),
        ],
      ),
    );
  }

  Widget _buildOrganizationSection(BuildContext context) {
    return SliverToBoxAdapter(
      child: Semantics(
        button: true,
        child: InkWell(
          onTap: () => OrganizationDetailScreen.showPreloaded(
            context,
            organization,
            sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore,
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                ExcludeSemantics(
                  child: OrganizationLogo(image: organization.logo, size: 32, fixedRadius: 8),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text.rich(
                        organization.displayName.l10nSpan(context),
                        textAlign: TextAlign.start,
                        style: context.textTheme.labelLarge,
                      ),
                      Text.rich(
                        organization.category?.l10nSpan(context) ?? ''.toTextSpan(context),
                        textAlign: TextAlign.start,
                        style: context.textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
                const Icon(Icons.chevron_right_rounded),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildAttributesSection(BuildContext context) {
    final attributesSliver = SliverList.separated(
      itemCount: cardRequests.length,
      itemBuilder: (context, i) {
        final entry = cardRequests[i];
        return SharedAttributesCard(
          card: entry.selection,
          onPressed: () => CheckAttributesScreen.show(
            context,
            card: entry.selection,
            onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
          ),
        );
      },
      separatorBuilder: (context, i) => const SizedBox(height: 16),
    );
    final headerSliver = SliverList.list(
      children: [
        const Align(
          alignment: Alignment.centerLeft,
          child: Icon(Icons.credit_card_outlined),
        ),
        const SizedBox(height: 16),
        TitleText(
          context.l10n.loginDetailScreenCredentialsTitle,
          style: context.textTheme.titleLarge,
        ),
        const SizedBox(height: 8),
        BodyText(context.l10n.loginDetailScreenCredentialsBody),
        const SizedBox(height: 24),
      ],
    );

    return SliverPadding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      sliver: SliverMainAxisGroup(
        slivers: [
          headerSliver,
          attributesSliver,
        ],
      ),
    );
  }

  Widget _buildAgreementSection(BuildContext context) {
    return SliverPadding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      sliver: SliverList.list(
        children: [
          const Align(
            alignment: Alignment.centerLeft,
            child: Icon(Icons.handshake_outlined),
          ),
          const SizedBox(height: 16),
          TitleText(
            context.l10n.loginDetailScreenAgreementTitle,
            style: context.textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          BodyText(
            context.read<ContextMapper<OrganizationPolicy, String>>().map(
                  context,
                  OrganizationPolicy(organization: organization, policy: policy),
                ),
          ),
          const SizedBox(height: 6),
          LinkButton(
            text: Text.rich(context.l10n.loginDetailScreenAgreementCta.toTextSpan(context)),
            onPressed: () => PolicyScreen.show(context, organization, policy),
          ),
        ],
      ),
    );
  }

  static Future<void> show(
    BuildContext context,
    Organization organization,
    Policy policy,
    List<DiscloseCardRequest> cardRequests, {
    required bool sharedDataWithOrganizationBefore,
    VoidCallback? onReportIssuePressed,
  }) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (context) {
          return LoginDetailScreen(
            organization: organization,
            policy: policy,
            cardRequests: cardRequests,
            sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore,
            onReportIssuePressed: onReportIssuePressed,
          );
        },
      ),
    );
  }
}
