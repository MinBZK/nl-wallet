import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/launch_util.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/text_with_link.dart';
import '../../common/widget/wallet_scrollbar.dart';

class OrganizationApprovePage extends StatelessWidget {
  /// Callback that is triggered when the user approves the request
  final VoidCallback onAcceptPressed;

  /// Callback that is triggered when the user declines the request
  final VoidCallback onDeclinePressed;

  /// Callback that is triggered when the user wants to report an issue
  final VoidCallback? onReportIssuePressed;

  /// Callback that is triggered when the user presses the button to view the organization details
  final VoidCallback onShowDetailsPressed;

  /// The organization that user is interacting with
  final Organization organization;

  /// The url from which the user should have opened the flow. Prominently displayed for the user to check
  final String originUrl;

  /// Tells the Page in which flow it's currently used, used to select the correct string resources
  final ApprovalPurpose purpose;

  /// If true, the 'first interaction' banner will be shown
  final bool sharedDataWithOrganizationBefore;

  /// If `crossDevice`, the 'fraud warning' (including `originUrl`) will be shown
  final DisclosureSessionType? sessionType;

  const OrganizationApprovePage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.onShowDetailsPressed,
    required this.organization,
    required this.originUrl,
    required this.purpose,
    this.onReportIssuePressed,
    this.sharedDataWithOrganizationBefore = false,
    this.sessionType,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: CustomScrollView(
          restorationId: 'approve_organization_scrollview',
          slivers: <Widget>[
            const SliverSizedBox(height: 16),
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(
              child: ListButton(
                onPressed: onShowDetailsPressed,
                text: Text.rich(_moreInfoButtonText(context).toTextSpan(context)),
              ),
            ),
            const SliverSizedBox(height: 32),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  const Divider(height: 1),
                  ConfirmButtons(
                    primaryButton: PrimaryButton(
                      key: const Key('acceptButton'),
                      onPressed: onAcceptPressed,
                      text: Text.rich(_approveButtonText(context).toTextSpan(context)),
                      icon: Icon(_primaryIcon()),
                    ),
                    secondaryButton: SecondaryButton(
                      key: const Key('rejectButton'),
                      onPressed: onDeclinePressed,
                      text: Text.rich(_declineButtonText(context).toTextSpan(context)),
                      icon: const Icon(Icons.block_flipped),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  IconData _primaryIcon() {
    return switch (purpose) {
      ApprovalPurpose.issuance => Icons.arrow_forward,
      ApprovalPurpose.disclosure => Icons.arrow_forward,
      ApprovalPurpose.sign => Icons.arrow_forward,
      ApprovalPurpose.login => Icons.key_outlined,
    };
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          OrganizationLogo(
            image: organization.logo,
            size: 64,
            fixedRadius: 12,
          ),
          const SizedBox(height: 24),
          _buildHeaderTitleText(context),
          if (sessionType == DisclosureSessionType.crossDevice) ...[
            const SizedBox(height: 8),
            _buildFraudInfoText(context),
          ],
        ],
      ),
    );
  }

  Widget _buildHeaderTitleText(BuildContext context) {
    final title = switch (purpose) {
      ApprovalPurpose.issuance =>
        context.l10n.organizationApprovePageGenericTitle(organization.displayName.l10nValue(context)),
      ApprovalPurpose.disclosure =>
        context.l10n.organizationApprovePageGenericTitle(organization.displayName.l10nValue(context)),
      ApprovalPurpose.sign =>
        context.l10n.organizationApprovePageGenericTitle(organization.displayName.l10nValue(context)),
      ApprovalPurpose.login =>
        context.l10n.organizationApprovePageLoginTitle(organization.displayName.l10nValue(context)),
    };
    return TitleText(
      title,
      style: context.textTheme.displayMedium,
      textAlign: TextAlign.start,
    );
  }

  Widget _buildFraudInfoText(BuildContext context) {
    final fraudTextPart1 = context.l10n.organizationApprovePageFraudInfoPart1;
    final fraudTextPart2 = context.l10n.organizationApprovePageFraudInfoPart2(originUrl);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text.rich(
          fraudTextPart1.toTextSpan(context),
          textAlign: TextAlign.start,
          style: context.textTheme.bodyLarge,
        ), // Migrate to [BodyText] on merge with PVW-2501
        const SizedBox(height: 8),
        TextWithLink(
          fullText: fraudTextPart2,
          ctaText: originUrl,
          onTapHint: context.l10n.generalWCAGOpenLink,
          onCtaPressed: () => launchUrlStringCatching(originUrl),
        ),
      ],
    );
  }

  String _moreInfoButtonText(BuildContext context) {
    switch (purpose) {
      case ApprovalPurpose.login:
        return context.l10n.organizationApprovePageMoreInfoLoginCta;
      default:
        return context.l10n.organizationApprovePageMoreInfoCta;
    }
  }

  String _approveButtonText(BuildContext context) {
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return context.l10n.organizationApprovePageApproveCta;
      case ApprovalPurpose.disclosure:
        return context.l10n.organizationApprovePageShareWithApproveCta;
      case ApprovalPurpose.sign:
        return context.l10n.organizationApprovePageApproveCta;
      case ApprovalPurpose.login:
        return context.l10n.organizationApprovePageLoginCta;
    }
  }

  String _declineButtonText(BuildContext context) {
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return context.l10n.organizationApprovePageDenyCta;
      case ApprovalPurpose.disclosure:
        return context.l10n.organizationApprovePageShareWithDenyCta;
      case ApprovalPurpose.sign:
        return context.l10n.organizationApprovePageDenyCta;
      case ApprovalPurpose.login:
        return context.l10n.organizationApprovePageCancelLoginCta;
    }
  }
}

enum ApprovalPurpose { issuance, disclosure, sign, login }
