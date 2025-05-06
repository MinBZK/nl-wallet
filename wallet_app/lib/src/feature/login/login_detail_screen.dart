import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/card/wallet_card.dart';
import '../../domain/model/organization.dart';
import '../../domain/model/policy/organization_policy.dart';
import '../../domain/model/policy/policy.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/mapper/context_mapper.dart';
import '../check_attributes/check_attributes_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/card/shared_attributes_card.dart';
import '../common/widget/organization/organization_logo.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../info/info_screen.dart';
import '../organization/detail/organization_detail_screen.dart';
import '../policy/policy_screen.dart';
import 'argument/login_detail_screen_argument.dart';

class LoginDetailScreen extends StatelessWidget {
  final Organization organization;
  final Policy policy;
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
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
    required this.requestedAttributes,
    required this.sharedDataWithOrganizationBefore,
    this.onReportIssuePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
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
          SliverWalletAppBar(
            title: context.l10n.loginDetailScreenTitle(organization.displayName.l10nValue(context)),
            scrollController: PrimaryScrollController.maybeOf(context),
            actions: const [HelpIconButton()],
          ),
          const SliverSizedBox(height: 24),
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
    );
  }

  Widget _buildAttributesSection(BuildContext context) {
    final attributesSliver = SliverList.separated(
      itemCount: requestedAttributes.length,
      itemBuilder: (context, i) {
        final entry = requestedAttributes.entries.elementAt(i);
        return SharedAttributesCard(
          card: entry.key,
          attributes: entry.value,
          onTap: () => CheckAttributesScreen.show(
            context,
            card: entry.key,
            attributes: entry.value,
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
        SizedBox(height: 8),
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
          SizedBox(height: 8),
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
    Map<WalletCard, List<DataAttribute>> requestedAttributes, {
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
            requestedAttributes: requestedAttributes,
            sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore,
            onReportIssuePressed: onReportIssuePressed,
          );
        },
      ),
    );
  }
}
