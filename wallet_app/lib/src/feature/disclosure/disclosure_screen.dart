import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../organization/approve/organization_approve_page.dart';
import '../report_issue/report_issue_screen.dart';
import 'bloc/disclosure_bloc.dart';
import 'page/disclosure_confirm_data_attributes_page.dart';
import 'page/disclosure_confirm_pin_page.dart';
import 'page/disclosure_generic_error_page.dart';
import 'page/disclosure_missing_attributes_page.dart';
import 'page/disclosure_report_submitted_page.dart';
import 'page/disclosure_stopped_page.dart';
import 'page/disclosure_success_page.dart';
import 'widget/disclosure_stop_sheet.dart';

class DisclosureScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the DisclosureScreen');
    }
  }

  const DisclosureScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'disclosure_scaffold',
      appBar: AppBar(
        title: Text(context.l10n.verificationScreenTitle),
        leading: _buildBackButton(context),
        actions: [_buildCloseButton(context)],
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<DisclosureBloc>();
          if (bloc.state.canGoBack) {
            bloc.add(const DisclosureBackPressed());
          } else {
            _stopDisclosure(context);
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
    return BlocBuilder<DisclosureBloc, DisclosureState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.read<DisclosureBloc>().add(const DisclosureBackPressed()),
        );
      },
    );
  }

  /// The close button stops/closes the disclosure flow.
  /// It is only visible in the semantics tree when the disclosure flow is in progress.
  Widget _buildCloseButton(BuildContext context) {
    final closeButton = CloseButton(onPressed: () => _stopDisclosure(context));
    return BlocBuilder<DisclosureBloc, DisclosureState>(
      builder: (context, state) {
        if (state.stepperProgress == 1.0) {
          return ExcludeSemantics(child: closeButton);
        } else {
          return closeButton;
        }
      },
    );
  }

  Widget _buildStepper() {
    return BlocBuilder<DisclosureBloc, DisclosureState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<DisclosureBloc, DisclosureState>(
      builder: (context, state) {
        Widget result = switch (state) {
          DisclosureInitial() => _buildLoading(),
          DisclosureLoadInProgress() => _buildLoading(),
          DisclosureCheckOrganization() => _buildCheckOrganizationPage(context, state),
          DisclosureMissingAttributes() => _buildMissingAttributesPage(context, state),
          DisclosureConfirmDataAttributes() => _buildConfirmDataAttributesPage(context, state),
          DisclosureConfirmPin() => _buildConfirmPinPage(context, state),
          DisclosureStopped() => _buildStoppedPage(context, state),
          DisclosureLeftFeedback() => _buildLeftFeedbackPage(context, state),
          DisclosureSuccess() => _buildSuccessPage(context, state),
          DisclosureGenericError() => _buildGenericErrorPage(context, state),
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

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildCheckOrganizationPage(BuildContext context, DisclosureCheckOrganization state) {
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopDisclosure(context),
      onAcceptPressed: () => context.read<DisclosureBloc>().add(const DisclosureOrganizationApproved()),
      organization: state.flow.organization,
      isFirstInteractionWithOrganization: !state.flow.hasPreviouslyInteractedWithOrganization,
      purpose: ApprovalPurpose.disclosure,
      requestPurpose: state.flow.requestPurpose,
      onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
    );
  }

  Widget _buildMissingAttributesPage(BuildContext context, DisclosureMissingAttributes state) {
    return DisclosureMissingAttributesPage(
      onDecline: () => context.read<DisclosureBloc>().add(DisclosureStopRequested(flow: state.flow)),
      flow: state.flow,
    );
  }

  Widget _buildConfirmDataAttributesPage(BuildContext context, DisclosureConfirmDataAttributes state) {
    return DisclosureConfirmDataAttributesPage(
      onDeclinePressed: () => _stopDisclosure(context),
      onAcceptPressed: () => context.read<DisclosureBloc>().add(const DisclosureShareRequestedAttributesApproved()),
      onReportIssuePressed: () => _onReportIssuePressed(context, _resolveReportingOptionsForState(context)),
      flow: state.flow,
    );
  }

  Widget _buildConfirmPinPage(BuildContext context, DisclosureConfirmPin state) {
    return DisclosureConfirmPinPage(
      onPinValidated: () => context.read<DisclosureBloc>().add(DisclosurePinConfirmed(state.flow)),
    );
  }

  Widget _buildStoppedPage(BuildContext context, DisclosureStopped state) {
    return DisclosureStoppedPage(onClosePressed: () => Navigator.pop(context));
  }

  Widget _buildLeftFeedbackPage(BuildContext context, DisclosureLeftFeedback state) {
    return DisclosureReportSubmittedPage(
      onClosePressed: () => Navigator.pop(context),
    );
  }

  Widget _buildSuccessPage(BuildContext context, DisclosureSuccess state) {
    return DisclosureSuccessPage(
      verifierShortName: state.flow.organization.shortName,
      onClosePressed: () => Navigator.pop(context),
      onHistoryPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
    );
  }

  Widget _buildGenericErrorPage(BuildContext context, DisclosureGenericError state) {
    return DisclosureGenericErrorPage(
      onClosePressed: () => Navigator.pop(context),
    );
  }

  void _stopDisclosure(BuildContext context) async {
    final bloc = context.read<DisclosureBloc>();
    if (bloc.state.showStopConfirmation) {
      final availableReportOptions = _resolveReportingOptionsForState(context);
      final organizationName = context.read<DisclosureBloc>().state.organization?.shortName ?? '-';
      final stopPressed = await DisclosureStopSheet.show(
        context,
        organizationName: organizationName,
        onReportIssuePressed: availableReportOptions.isEmpty
            ? null
            : () {
                Navigator.pop(context); //Close the StopDisclosureSheet
                _onReportIssuePressed(context, availableReportOptions);
              },
      );
      if (stopPressed) bloc.add(DisclosureStopRequested(flow: bloc.state.flow));
    } else {
      Navigator.pop(context);
    }
  }

  void _onReportIssuePressed(BuildContext context, List<ReportingOption> optionsToShow) async {
    final bloc = context.read<DisclosureBloc>();
    final selectedOption = await ReportIssueScreen.show(context, optionsToShow);
    if (selectedOption != null) {
      bloc.add(DisclosureReportPressed(flow: bloc.state.flow, option: selectedOption));
    }
  }

  List<ReportingOption> _resolveReportingOptionsForState(BuildContext context) {
    final state = context.read<DisclosureBloc>().state;
    if (state is DisclosureCheckOrganization) {
      return [
        ReportingOption.unknownOrganization,
        ReportingOption.requestNotInitiated,
        ReportingOption.suspiciousOrganization,
        ReportingOption.impersonatingOrganization,
      ];
    } else if (state is DisclosureConfirmDataAttributes || state is DisclosureConfirmPin) {
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
