import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../navigation/wallet_routes.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../organization/approve/organization_approve_page.dart';
import '../report_issue/report_issue_screen.dart';
import 'bloc/verification_bloc.dart';
import 'page/verification_confirm_data_attributes_page.dart';
import 'page/verification_confirm_pin_page.dart';
import 'page/verification_generic_error_page.dart';
import 'page/verification_missing_attributes_page.dart';
import 'page/verification_report_submitted_page.dart';
import 'page/verification_stopped_page.dart';
import 'page/verification_success_page.dart';
import 'widget/verification_stop_sheet.dart';

class VerificationScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the VerificationScreen');
    }
  }

  const VerificationScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'verification_scaffold',
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).verificationScreenTitle),
        leading: _buildBackButton(context),
        actions: [CloseButton(onPressed: () => _stopVerification(context))],
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<VerificationBloc>();
          if (bloc.state.canGoBack) {
            bloc.add(const VerificationBackPressed());
          } else {
            _stopVerification(context);
          }
          return false;
        },
        child: Column(
          children: [
            _buildStepper(),
            Expanded(
              child: SafeArea(
                child: _buildPage(),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<VerificationBloc, VerificationState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.read<VerificationBloc>().add(const VerificationBackPressed()),
        );
      },
    );
  }

  Widget _buildStepper() {
    return BlocBuilder<VerificationBloc, VerificationState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<VerificationBloc, VerificationState>(
      builder: (context, state) {
        Widget? result;
        if (state is VerificationInitial) result = _buildLoading();
        if (state is VerificationLoadInProgress) result = _buildLoading();
        if (state is VerificationCheckOrganization) result = _buildCheckOrganizationPage(context, state);
        if (state is VerificationMissingAttributes) result = _buildMissingAttributesPage(context, state);
        if (state is VerificationConfirmDataAttributes) result = _buildConfirmDataAttributesPage(context, state);
        if (state is VerificationConfirmPin) result = _buildConfirmPinPage(context, state);
        if (state is VerificationStopped) result = _buildStoppedPage(context, state);
        if (state is VerificationLeftFeedback) result = _buildLeftFeedbackPage(context, state);
        if (state is VerificationSuccess) result = _buildSuccessPage(context, state);
        if (state is VerificationGenericError) result = _buildGenericErrorPage(context, state);
        if (result == null) throw UnsupportedError('Unknown state: $state');

        final skipAnim = !state.didGoBack && state is VerificationCheckOrganization;
        return FakePagingAnimatedSwitcher(
          animateBackwards: state.didGoBack,
          animate: !skipAnim,
          child: result,
        );
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildCheckOrganizationPage(BuildContext context, VerificationCheckOrganization state) {
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopVerification(context),
      onAcceptPressed: () => context.read<VerificationBloc>().add(const VerificationOrganizationApproved()),
      organization: state.flow.organization,
      isFirstInteractionWithOrganization: !state.flow.hasPreviouslyInteractedWithOrganization,
      purpose: ApprovalPurpose.verification,
      requestPurpose: state.flow.requestPurpose,
      onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
    );
  }

  Widget _buildMissingAttributesPage(BuildContext context, VerificationMissingAttributes state) {
    return VerificationMissingAttributesPage(
      onDecline: () => context.read<VerificationBloc>().add(VerificationStopRequested(flow: state.flow)),
      flow: state.flow,
    );
  }

  Widget _buildConfirmDataAttributesPage(BuildContext context, VerificationConfirmDataAttributes state) {
    return VerificationConfirmDataAttributesPage(
      onDeclinePressed: () => _stopVerification(context),
      onAcceptPressed: () => context.read<VerificationBloc>().add(const VerificationShareRequestedAttributesApproved()),
      onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
      flow: state.flow,
    );
  }

  Widget _buildConfirmPinPage(BuildContext context, VerificationConfirmPin state) {
    return VerificationConfirmPinPage(
      onPinValidated: () => context.read<VerificationBloc>().add(VerificationPinConfirmed(state.flow)),
    );
  }

  Widget _buildStoppedPage(BuildContext context, VerificationStopped state) {
    return VerificationStoppedPage(onClosePressed: () => Navigator.pop(context));
  }

  Widget _buildLeftFeedbackPage(BuildContext context, VerificationLeftFeedback state) {
    return VerificationReportSubmittedPage(
      onClosePressed: () => Navigator.pop(context),
    );
  }

  Widget _buildSuccessPage(BuildContext context, VerificationSuccess state) {
    return VerificationSuccessPage(
      verifierShortName: state.flow.organization.shortName,
      onClosePressed: () => Navigator.pop(context),
      onHistoryPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
    );
  }

  Widget _buildGenericErrorPage(BuildContext context, VerificationGenericError state) {
    return VerificationGenericErrorPage(
      onClosePressed: () => Navigator.pop(context),
    );
  }

  void _stopVerification(BuildContext context) async {
    final bloc = context.read<VerificationBloc>();
    if (bloc.state.showStopConfirmation) {
      final availableReportOptions = _resolveReportingOptionsForState(context);
      final organizationName = context.read<VerificationBloc>().state.organization?.shortName ?? '-';
      final stopPressed = await VerificationStopSheet.show(
        context,
        organizationName: organizationName,
        onReportIssuePressed: availableReportOptions.isEmpty
            ? null
            : () {
                Navigator.pop(context); //Close the StopVerificationSheet
                _onReportIssuePressed(context, availableReportOptions);
              },
      );
      if (stopPressed) bloc.add(VerificationStopRequested(flow: bloc.state.flow));
    } else {
      Navigator.pop(context);
    }
  }

  void _onReportIssuePressed(BuildContext context, List<ReportingOption> optionsToShow) async {
    final bloc = context.read<VerificationBloc>();
    final selectedOption = await ReportIssueScreen.show(context, optionsToShow);
    if (selectedOption != null) {
      bloc.add(VerificationReportPressed(flow: bloc.state.flow, option: selectedOption));
    }
  }

  List<ReportingOption> _resolveReportingOptionsForState(BuildContext context) {
    final state = context.read<VerificationBloc>().state;
    if (state is VerificationCheckOrganization) {
      return [
        ReportingOption.unknownOrganization,
        ReportingOption.requestNotInitiated,
        ReportingOption.suspiciousOrganization,
        ReportingOption.impersonatingOrganization,
      ];
    } else if (state is VerificationConfirmDataAttributes || state is VerificationConfirmPin) {
      return [
        ReportingOption.untrusted,
        ReportingOption.overAskingOrganization,
        ReportingOption.suspiciousOrganization,
        ReportingOption.unreasonableTerms,
      ];
    }
    return <ReportingOption>[];
  }
}
