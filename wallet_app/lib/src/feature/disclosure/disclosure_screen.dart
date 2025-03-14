import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../domain/usecase/disclosure/accept_disclosure_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/localized_text_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/launch_util.dart';
import '../common/dialog/scan_with_wallet_dialog.dart';
import '../common/page/generic_loading_page.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import '../history/detail/argument/history_detail_screen_argument.dart';
import '../login/login_detail_screen.dart';
import '../organization/approve/organization_approve_page.dart';
import '../organization/detail/organization_detail_screen.dart';
import '../pin/bloc/pin_bloc.dart';
import '../report_issue/report_issue_screen.dart';
import 'argument/disclosure_screen_argument.dart';
import 'bloc/disclosure_bloc.dart';
import 'page/disclosure_confirm_data_attributes_page.dart';
import 'page/disclosure_confirm_pin_page.dart';
import 'page/disclosure_generic_error_page.dart';
import 'page/disclosure_missing_attributes_page.dart';
import 'page/disclosure_network_error_page.dart';
import 'page/disclosure_report_submitted_page.dart';
import 'page/disclosure_stopped_page.dart';
import 'page/disclosure_success_page.dart';
import 'widget/disclosure_stop_sheet.dart';

class DisclosureScreen extends StatelessWidget {
  static DisclosureScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return tryCast<DisclosureScreenArgument>(args) ?? DisclosureScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [DisclosureScreenArgument] when opening the DisclosureScreen');
    }
  }

  const DisclosureScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final progress = context.watch<DisclosureBloc>().state.stepperProgress;
    return ScrollOffsetProvider(
      debugLabel: 'disclosure',
      child: Scaffold(
        restorationId: 'disclosure_scaffold',
        appBar: WalletAppBar(
          leading: _buildBackButton(context),
          automaticallyImplyLeading: false,
          actions: [
            const HelpIconButton(),
            CloseIconButton(
              onPressed: () => _stopDisclosure(context),
            ),
          ],
          title: _buildTitle(context),
          progress: progress,
        ),
        body: PopScope(
          canPop: false,
          onPopInvokedWithResult: (didPop, result) {
            if (didPop) {
              return;
            }
            final bloc = context.bloc;
            if (bloc.state.canGoBack) {
              bloc.add(const DisclosureBackPressed());
            } else {
              _stopDisclosure(context);
            }
          },
          child: _buildPage(),
        ),
      ),
    );
  }

  Widget? _buildBackButton(BuildContext context) {
    final canGoBack = context.watch<DisclosureBloc>().state.canGoBack;
    if (!canGoBack) return null;
    return BackIconButton(
      onPressed: () => context.bloc.add(const DisclosureBackPressed()),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<DisclosureBloc, DisclosureState>(
      /// Reset the [ScrollOffset] used by [FadeInAtOffset] when the state (and thus the visible page) changes.
      listener: (context, state) {
        context.read<ScrollOffset>().offset = 0;
        if (state is DisclosureExternalScannerError) {
          Navigator.maybePop(context).then((popped) {
            // ignore: use_build_context_synchronously
            ScanWithWalletDialog.show(context);
          });
        }
      },
      builder: (context, state) {
        final Widget result = switch (state) {
          DisclosureInitial() => _buildInitialLoading(context),
          DisclosureLoadInProgress() => _buildLoading(),
          DisclosureCheckOrganization() => _buildCheckOrganizationPage(context, state),
          DisclosureCheckOrganizationForLogin() => _buildCheckOrganizationForLoginPage(context, state),
          DisclosureMissingAttributes() => _buildMissingAttributesPage(context, state),
          DisclosureConfirmDataAttributes() => _buildConfirmDataAttributesPage(context, state),
          DisclosureConfirmPin() => _buildConfirmPinPage(context, state),
          DisclosureStopped() => _buildStoppedPage(context, state),
          DisclosureLeftFeedback() => _buildLeftFeedbackPage(context, state),
          DisclosureSuccess() => _buildSuccessPage(context, state),
          DisclosureNetworkError() => _buildNetworkErrorPage(context, state),
          DisclosureGenericError() => _buildGenericErrorPage(context, returnUrl: state.returnUrl),
          DisclosureSessionExpired() => _buildSessionExpiredPage(context, state),
          DisclosureExternalScannerError() => _buildGenericErrorPage(context),
          DisclosureCancelledSessionError() => _buildCancelledSessionPage(context, state),
        };

        final skipAnim = !state.didGoBack && state is DisclosureCheckOrganization;
        return FakePagingAnimatedSwitcher(
          animateBackwards: state.didGoBack,
          animate: !skipAnim,
          child: result,
        );
      },
    );
  }

  Widget _buildInitialLoading(BuildContext context) => GenericLoadingPage(
        title: context.l10n.disclosureLoadingTitle,
        description: context.l10n.disclosureLoadingSubtitle,
      );

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildCheckOrganizationPage(BuildContext context, DisclosureCheckOrganization state) {
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopDisclosure(context),
      onAcceptPressed: () => context.bloc.add(const DisclosureOrganizationApproved()),
      organization: state.relyingParty,
      originUrl: state.originUrl,
      sharedDataWithOrganizationBefore: state.sharedDataWithOrganizationBefore,
      sessionType: state.sessionType,
      purpose: ApprovalPurpose.disclosure,
      onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
      onShowDetailsPressed: () {
        OrganizationDetailScreen.showPreloaded(
          context,
          state.relyingParty,
          sharedDataWithOrganizationBefore: state.sharedDataWithOrganizationBefore,
          onReportIssuePressed: () {
            _onReportIssuePressed(context, _resolveReportingOptionsForState(context));
          },
        );
      },
    );
  }

  Widget _buildCheckOrganizationForLoginPage(BuildContext context, DisclosureCheckOrganizationForLogin state) {
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopDisclosure(context),
      onAcceptPressed: () => context.bloc.add(const DisclosureOrganizationApproved()),
      organization: state.relyingParty,
      originUrl: state.originUrl,
      sessionType: state.sessionType,
      purpose: ApprovalPurpose.login,
      onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
      onShowDetailsPressed: () {
        LoginDetailScreen.show(
          context,
          state.relyingParty,
          state.policy,
          state.requestedAttributes,
          onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
          sharedDataWithOrganizationBefore: state.sharedDataWithOrganizationBefore,
        );
      },
    );
  }

  Widget _buildMissingAttributesPage(BuildContext context, DisclosureMissingAttributes state) {
    return DisclosureMissingAttributesPage(
      onDecline: () => context.bloc.add(const DisclosureStopRequested()),
      missingAttributes: state.missingAttributes,
      organization: state.relyingParty,
      onReportIssuePressed: () {
        final options = _resolveReportingOptionsForState(context);
        _onReportIssuePressed(context, options);
      },
    );
  }

  Widget _buildConfirmDataAttributesPage(BuildContext context, DisclosureConfirmDataAttributes state) {
    return DisclosureConfirmDataAttributesPage(
      onDeclinePressed: () => _stopDisclosure(context),
      onAcceptPressed: () => context.bloc.add(const DisclosureShareRequestedAttributesApproved()),
      relyingParty: state.relyingParty,
      requestedAttributes: state.requestedAttributes,
      policy: state.policy,
      requestPurpose: state.requestPurpose,
    );
  }

  Widget _buildConfirmPinPage(BuildContext context, DisclosureConfirmPin state) {
    final title = state.isLoginFlow
        ? context.l10n.disclosureConfirmPinPageForLoginTitle
        : context.l10n.disclosureConfirmPinPageTitle;
    return DisclosureConfirmPinPage(
      title: title,
      bloc: PinBloc(context.read<AcceptDisclosureUseCase>()),
      onPinValidated: (returnUrl) => context.bloc.add(DisclosurePinConfirmed(returnUrl: returnUrl)),
      onConfirmWithPinFailed: (context, state) => context.bloc.add(DisclosureConfirmPinFailed(error: state.error)),
    );
  }

  Widget _buildStoppedPage(BuildContext context, DisclosureStopped state) {
    return DisclosureStoppedPage(
      organization: state.organization,
      onClosePressed: (returnUrl) {
        Navigator.pop(context);
        returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
      },
      isLoginFlow: state.isLoginFlow,
      returnUrl: state.returnUrl,
    );
  }

  Widget _buildLeftFeedbackPage(BuildContext context, DisclosureLeftFeedback state) {
    return DisclosureReportSubmittedPage(
      onClosePressed: () => Navigator.pop(context),
    );
  }

  Widget _buildSuccessPage(BuildContext context, DisclosureSuccess state) {
    return DisclosureSuccessPage(
      organizationDisplayName: state.relyingParty.displayName,
      returnUrl: state.returnUrl,
      isLoginFlow: state.isLoginFlow,
      onHistoryPressed: state.event == null
          ? null
          : () {
              Navigator.pushNamed(
                context,
                WalletRoutes.historyDetailRoute,
                arguments: HistoryDetailScreenArgument(walletEvent: state.event!).toMap(),
              );
            },
      onPrimaryPressed: (returnUrl) {
        Navigator.pop(context);
        returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
      },
    );
  }

  Widget _buildNetworkErrorPage(BuildContext context, DisclosureNetworkError state) {
    return DisclosureNetworkErrorPage(
      hasInternet: state.hasInternet,
      onStopPressed: () => Navigator.pop(context),
    );
  }

  Widget _buildGenericErrorPage(BuildContext context, {String? returnUrl}) {
    return DisclosureGenericErrorPage(
      onStopPressed: () {
        returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
        Navigator.pop(context);
      },
    );
  }

  Future<void> _stopDisclosure(BuildContext context) async {
    final bloc = context.bloc;
    if (bloc.state.showStopConfirmation) {
      final availableReportOptions = _resolveReportingOptionsForState(context);
      final organizationName = context.read<DisclosureBloc>().relyingParty?.displayName;
      final stopPressed = await DisclosureStopSheet.show(
        context,
        organizationName: organizationName,
        isLoginFlow: bloc.isLoginFlow,
        onReportIssuePressed: availableReportOptions.isEmpty
            ? null
            : () {
                Navigator.pop(context); //Close the StopDisclosureSheet
                _onReportIssuePressed(context, availableReportOptions);
              },
      );
      if (stopPressed) bloc.add(const DisclosureStopRequested());
    } else {
      Navigator.pop(context);
    }
  }

  Future<void> _onReportIssuePressed(BuildContext context, List<ReportingOption> optionsToShow) async {
    final bloc = context.bloc;
    final selectedOption = await ReportIssueScreen.show(context, optionsToShow);
    if (selectedOption != null) {
      bloc.add(DisclosureReportPressed(option: selectedOption));
    }
  }

  List<ReportingOption> _resolveReportingOptionsForState(BuildContext context) {
    final state = context.read<DisclosureBloc>().state;
    switch (state) {
      case DisclosureCheckOrganization():
      case DisclosureCheckOrganizationForLogin():
        return [
          ReportingOption.unknownOrganization,
          ReportingOption.requestNotInitiated,
          ReportingOption.suspiciousOrganization,
          ReportingOption.impersonatingOrganization,
        ];
      case DisclosureConfirmPin():
      case DisclosureConfirmDataAttributes():
        return [
          ReportingOption.untrusted,
          ReportingOption.overAskingOrganization,
          ReportingOption.suspiciousOrganization,
          ReportingOption.unreasonableTerms,
        ];
      case DisclosureMissingAttributes():
        return [
          ReportingOption.overAskingOrganization,
          ReportingOption.suspiciousOrganization,
        ];
      case DisclosureInitial():
      case DisclosureLoadInProgress():
      case DisclosureExternalScannerError():
      case DisclosureGenericError():
      case DisclosureSessionExpired():
      case DisclosureNetworkError():
      case DisclosureSuccess():
      case DisclosureStopped():
      case DisclosureCancelledSessionError():
      case DisclosureLeftFeedback():
        Fimber.d('No ReportingOptions provided for $state');
        return <ReportingOption>[];
    }
  }

  Widget _buildTitle(BuildContext context) {
    return BlocBuilder<DisclosureBloc, DisclosureState>(
      builder: (context, state) {
        final Widget? result = switch (state) {
          DisclosureInitial() => null,
          DisclosureLoadInProgress() => null,
          DisclosureCheckOrganization() => FadeInAtOffset(
              appearOffset: 120,
              visibleOffset: 150,
              child: Text.rich(
                context.l10n
                    .organizationApprovePageGenericTitle(
                      state.relyingParty.displayName.l10nValue(context),
                    )
                    .toTextSpan(context),
              ),
            ),
          DisclosureCheckOrganizationForLogin() => FadeInAtOffset(
              appearOffset: 120,
              visibleOffset: 150,
              child: Text.rich(
                context.l10n
                    .organizationApprovePageGenericTitle(
                      state.relyingParty.displayName.l10nValue(context),
                    )
                    .toTextSpan(context),
              ),
            ),
          DisclosureMissingAttributes() => null,
          DisclosureConfirmDataAttributes() => FadeInAtOffset(
              appearOffset: 70,
              visibleOffset: 100,
              child: Text.rich(
                context.l10n
                    .disclosureConfirmDataAttributesShareWithTitle(
                      state.relyingParty.displayName.l10nValue(context),
                    )
                    .toTextSpan(context),
              ),
            ),
          DisclosureConfirmPin() => null,
          DisclosureStopped() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text.rich(context.l10n.disclosureStoppedPageTitle.toTextSpan(context)),
            ),
          DisclosureLeftFeedback() => null,
          DisclosureSuccess() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text.rich(context.l10n.disclosureSuccessPageTitle.toTextSpan(context)),
            ),
          DisclosureNetworkError() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text(
                state.hasInternet ? context.l10n.errorScreenServerHeadline : context.l10n.errorScreenNoInternetHeadline,
              ),
            ),
          DisclosureExternalScannerError() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text.rich(context.l10n.disclosureGenericErrorPageTitle.toTextSpan(context)),
            ),
          DisclosureGenericError() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text.rich(context.l10n.disclosureGenericErrorPageTitle.toTextSpan(context)),
            ),
          DisclosureSessionExpired() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text.rich(context.l10n.errorScreenSessionExpiredHeadline.toTextSpan(context)),
            ),
          DisclosureCancelledSessionError() => FadeInAtOffset(
              appearOffset: 48,
              visibleOffset: 70,
              child: Text.rich(context.l10n.errorScreenCancelledSessionHeadline.toTextSpan(context)),
            ),
        };

        return result ?? const SizedBox.shrink();
      },
    );
  }

  Widget _buildSessionExpiredPage(BuildContext context, DisclosureSessionExpired state) {
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

  Widget _buildCancelledSessionPage(BuildContext context, DisclosureCancelledSessionError state) {
    return ErrorPage.cancelledSession(
      context,
      organizationName: state.relyingParty.displayName.l10nValue(context),
      onPrimaryActionPressed: () {
        Navigator.pop(context);
        state.returnUrl?.let((url) => launchUrlStringCatching(url, mode: LaunchMode.externalApplication));
      },
    );
  }
}

extension _DisclosureScreenExtensions on BuildContext {
  DisclosureBloc get bloc => read<DisclosureBloc>();
}
