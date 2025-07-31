import 'package:collection/collection.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/launch_util.dart';
import '../../wallet_assets.dart';
import '../common/dialog/scan_with_wallet_dialog.dart';
import '../common/page/generic_loading_page.dart';
import '../common/page/missing_attributes_page.dart';
import '../common/page/network_error_page.dart';
import '../common/page/terminal_page.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/wallet_app_bar.dart';
import '../dashboard/dashboard_screen.dart';
import '../error/error_page.dart';
import '../organization/approve/organization_approve_page.dart';
import '../report_issue/report_issue_screen.dart';
import 'argument/issuance_screen_argument.dart';
import 'bloc/issuance_bloc.dart';
import 'issuance_request_details_screen.dart';
import 'issuance_stop_sheet.dart';
import 'page/issuance_confirm_pin_for_disclosure_page.dart';
import 'page/issuance_confirm_pin_for_issuance_page.dart';
import 'page/issuance_generic_error_page.dart';
import 'page/issuance_relying_party_error_page.dart';
import 'page/issuance_review_cards_page.dart';
import 'page/issuance_stopped_page.dart';
import 'page/issuance_success_page.dart';

class IssuanceScreen extends StatelessWidget {
  static IssuanceScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return tryCast<IssuanceScreenArgument>(args) ?? IssuanceScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [IssuanceScreenArgument] when opening the IssuanceScreen');
    }
  }

  const IssuanceScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final progress = context.watch<IssuanceBloc>().state.stepperProgress;
    return Scaffold(
      restorationId: 'issuance_scaffold',
      appBar: WalletAppBar(
        leading: _buildBackButton(context),
        automaticallyImplyLeading: false,
        actions: [
          const HelpIconButton(),
          CloseIconButton(onPressed: () => _stopIssuance(context)),
        ],
        progress: progress,
      ),
      body: PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, result) {
          if (didPop) return;
          if (context.bloc.state.canGoBack) {
            context.bloc.add(const IssuanceBackPressed());
          } else {
            _stopIssuance(context);
          }
        },
        child: SafeArea(
          child: _buildPage(),
        ),
      ),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<IssuanceBloc, IssuanceState>(
      listener: (context, state) {
        if (state is IssuanceExternalScannerError) {
          Navigator.maybePop(context).then((popped) {
            if (context.mounted) ScanWithWalletDialog.show(context);
          });
        }
      },
      builder: (context, state) {
        final Widget result = switch (state) {
          IssuanceInitial() => _buildLoadingRequestPage(context),
          IssuanceLoadInProgress() => _buildLoadingCardsPage(context),
          IssuanceCheckOrganization() => _buildCheckOrganizationPage(context, state),
          IssuanceMissingAttributes() => _buildMissingAttributes(context, state),
          IssuanceReviewCards() => _buildReviewCardsPage(context, state),
          IssuanceCompleted() => _buildIssuanceCompletedPage(context, state),
          IssuanceStopped() => _buildStoppedPage(context, state),
          IssuanceGenericError() => _buildGenericErrorPage(context),
          IssuanceProvidePinForDisclosure() => _buildProvidePinForDisclosurePage(context),
          IssuanceProvidePinForIssuance() => _buildProvidePinForIssuancePage(context, state),
          IssuanceNoCardsRetrieved() => _buildNoCardsReceived(context, state),
          IssuanceExternalScannerError() => _buildGenericErrorPage(context),
          IssuanceNetworkError() => _buildNetworkErrorPage(context, state),
          IssuanceSessionExpired() => _buildSessionExpiredPage(context, state),
          IssuanceSessionCancelled() => _buildCancelledSessionPage(context, state),
          IssuanceRelyingPartyError() => _buildRelyingPartyErrorPage(context, state),
        };

        final skipAnim = !state.didGoBack && state is IssuanceCheckOrganization;
        return FakePagingAnimatedSwitcher(
          animateBackwards: state.didGoBack,
          animate: !skipAnim,
          child: result,
        );
      },
    );
  }

  Widget _buildNetworkErrorPage(BuildContext context, IssuanceNetworkError state) {
    return NetworkErrorPage(
      hasInternet: state.hasInternet,
      onStopPressed: () => Navigator.pop(context),
    );
  }

  Widget _buildLoadingRequestPage(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.issuanceLoadingRequestTitle,
      description: context.l10n.issuanceLoadingRequestDescription,
    );
  }

  Widget _buildLoadingCardsPage(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.issuanceLoadingCardsTitle,
      description: context.l10n.issuanceLoadingCardsDescription,
    );
  }

  Widget? _buildBackButton(BuildContext context) {
    final canGoBack = context.watch<IssuanceBloc>().state.canGoBack;
    if (!canGoBack) return null;
    return BackIconButton(
      onPressed: () => context.bloc.add(const IssuanceBackPressed()),
    );
  }

  Widget _buildCheckOrganizationPage(BuildContext context, IssuanceCheckOrganization state) {
    late String description;
    final attributes = state.cardRequests.map((it) => it.selection.attributes).flattened;
    if (attributes.length == 1) {
      final requestedAttribute = attributes.firstOrNull;
      final attributeLabel = requestedAttribute?.label.l10nValue(context) ?? '';
      description = context.l10n.issuanceRequestedAttributeDescription(
        attributeLabel,
        state.organization.displayName.l10nValue(context),
      );
    } else {
      description = context.l10n.issuanceRequestedAttributesDescription(
        attributes.length,
        state.organization.displayName.l10nValue(context),
      );
    }
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopIssuance(context),
      originUrl: 'http://issue.origin.org',
      onAcceptPressed: () => context.bloc.add(const IssuanceOrganizationApproved()),
      organization: state.organization,
      purpose: ApprovalPurpose.issuance,
      description: description,
      onShowDetailsPressed: () => IssuanceRequestDetailsScreen.show(context, bloc: context.bloc),
    );
  }

  Widget _buildMissingAttributes(BuildContext context, IssuanceMissingAttributes state) {
    return MissingAttributesPage(
      organization: state.organization,
      onClosePressed: () => _stopIssuance(context),
      missingAttributes: state.missingAttributes,
      hasReturnUrl: false,
    );
  }

  Widget _buildProvidePinForDisclosurePage(BuildContext context) {
    return IssuanceConfirmPinForDisclosurePage(
      onPinValidated: (cards) => context.bloc.add(IssuancePinForDisclosureConfirmed(cards: cards)),
      onConfirmWithPinFailed: (context, errorState) => context.bloc.add(
        IssuanceConfirmPinFailed(error: errorState.error),
      ),
    );
  }

  Widget _buildProvidePinForIssuancePage(BuildContext context, IssuanceProvidePinForIssuance state) {
    return IssuanceConfirmPinForIssuancePage(
      onPinValidated: (_) => context.bloc.add(const IssuancePinForIssuanceConfirmed()),
      onConfirmWithPinFailed: (context, errorState) => context.bloc.add(
        IssuanceConfirmPinFailed(error: errorState.error),
      ),
      cards: state.cards,
    );
  }

  Widget _buildReviewCardsPage(BuildContext context, IssuanceReviewCards state) {
    return IssuanceReviewCardsPage(
      offeredCards: state.offeredCards,
      renewedCards: state.renewedCards,
      onAccept: (acceptedCards) => context.bloc.add(IssuanceApproveCards(cards: acceptedCards)),
      onDecline: () => _stopIssuance(context),
    );
  }

  Widget _buildIssuanceCompletedPage(BuildContext context, IssuanceCompleted state) {
    return IssuanceSuccessPage(
      onClose: () => DashboardScreen.show(context),
      cards: state.addedCards,
    );
  }

  Widget _buildStoppedPage(BuildContext context, IssuanceStopped state) {
    return IssuanceStoppedPage(
      onClosePressed: (returnUrl) {
        Navigator.pop(context);
        returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
      },
      onGiveFeedbackPressed: () => PlaceholderScreen.showGeneric(context),
      returnUrl: state.returnUrl,
    );
  }

  Widget _buildGenericErrorPage(BuildContext context, {String? returnUrl}) {
    return IssuanceGenericErrorPage(
      onClosePressed: () {
        returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
        Navigator.pop(context);
      },
    );
  }

  Future<void> _stopIssuance(BuildContext context) async {
    final bloc = context.bloc;
    if (bloc.state.showStopConfirmation) {
      final stopped = await IssuanceStopSheet.show(
        context,
        organizationName: bloc.relyingParty?.displayName.l10nValue(context),
        onReportIssuePressed: () => ReportIssueScreen.show(
          context,
          [ReportingOption.untrusted, ReportingOption.unreasonableTerms, ReportingOption.overAskingOrganization],
        ),
      );
      if (stopped) bloc.add(const IssuanceStopRequested());
    } else {
      Navigator.pop(context);
    }
  }

  Widget _buildNoCardsReceived(BuildContext context, IssuanceNoCardsRetrieved state) {
    return TerminalPage(
      title: context.l10n.issuanceNoCardsPageTitle,
      description: context.l10n.issuanceNoCardsPageDescription(state.organization.displayName.l10nValue(context)),
      primaryButtonCta: context.l10n.generalClose,
      onPrimaryPressed: () => _stopIssuance(context),
      illustration: const PageIllustration(asset: WalletAssets.svg_no_cards),
    );
  }

  Widget _buildSessionExpiredPage(BuildContext context, IssuanceSessionExpired state) {
    final userShouldRetryScan = state.isCrossDevice && state.canRetry;
    final hasReturnUrl = state.returnUrl != null;
    final userShouldRedirectToReturnUrl = !state.isCrossDevice && hasReturnUrl;
    String? cta;
    if (userShouldRetryScan) {
      cta = context.l10n.errorScreenSessionExpiredCrossDeviceCta;
    } else if (userShouldRedirectToReturnUrl) {
      cta = context.l10n.errorScreenSessionExpiredReturnUrlCta;
    }
    return ErrorPage.sessionExpired(
      context,
      style: state.canRetry ? ErrorCtaStyle.retry : ErrorCtaStyle.close,
      cta: cta,
      onPrimaryActionPressed: () {
        if (userShouldRetryScan) {
          Navigator.popUntil(context, ModalRoute.withName(WalletRoutes.dashboardRoute));
          Navigator.pushNamed(context, WalletRoutes.qrRoute);
        } else if (hasReturnUrl) {
          Navigator.maybePop(context);
          launchUrlStringCatching(state.returnUrl!, mode: LaunchMode.externalApplication);
        } else {
          Navigator.maybePop(context);
        }
      },
    );
  }

  Widget _buildCancelledSessionPage(BuildContext context, IssuanceSessionCancelled state) {
    return ErrorPage.cancelledSession(
      context,
      organizationName: state.relyingParty?.displayName.l10nValue(context) ?? context.l10n.organizationFallbackName,
      onPrimaryActionPressed: () {
        Navigator.pop(context);
        state.returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
      },
    );
  }

  Widget _buildRelyingPartyErrorPage(BuildContext context, IssuanceRelyingPartyError state) {
    return IssuanceRelyingPartyErrorPage(
      organizationName: state.organizationName?.l10nValue(context),
      onClosePressed: () => Navigator.pop(context),
    );
  }
}

extension _IssuanceScreenExtensions on BuildContext {
  IssuanceBloc get bloc => read<IssuanceBloc>();
}
