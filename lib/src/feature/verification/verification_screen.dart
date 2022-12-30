import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_routes.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/confirm_action_sheet.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/placeholder_screen.dart';
import '../organization/approve_organization_page.dart';
import 'bloc/verification_bloc.dart';
import 'page/verification_confirm_data_attributes_page.dart';
import 'page/verification_confirm_pin_page.dart';
import 'page/verification_generic_error_page.dart';
import 'page/verification_missing_attributes_page.dart';
import 'page/verification_stopped_page.dart';
import 'page/verification_success_page.dart';

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
            Expanded(child: _buildPage()),
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
        if (state is VerificationSuccess) result = _buildSuccessPage(context, state);
        if (state is VerificationGenericError) result = _buildGenericErrorPage(context, state);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildCheckOrganizationPage(BuildContext context, VerificationCheckOrganization state) {
    return ApproveOrganizationPage(
      onDecline: () => _stopVerification(context),
      onAccept: () => context.read<VerificationBloc>().add(const VerificationOrganizationApproved()),
      organization: state.flow.organization,
      purpose: ApprovalPurpose.verification,
    );
  }

  Widget _buildMissingAttributesPage(BuildContext context, VerificationMissingAttributes state) {
    return VerificationMissingAttributesPage(
      onDecline: () => context.read<VerificationBloc>().add(VerificationStopRequested(state.flow)),
      flow: state.flow,
    );
  }

  Widget _buildConfirmDataAttributesPage(BuildContext context, VerificationConfirmDataAttributes state) {
    return VerificationConfirmDataAttributesPage(
      onDecline: () => _stopVerification(context),
      onAccept: () => context.read<VerificationBloc>().add(const VerificationShareRequestedAttributesApproved()),
      flow: state.flow,
    );
  }

  Widget _buildConfirmPinPage(BuildContext context, VerificationConfirmPin state) {
    return VerificationConfirmPinPage(
      onPinValidated: () => context.read<VerificationBloc>().add(VerificationPinConfirmed(state.flow)),
    );
  }

  Widget _buildStoppedPage(BuildContext context, VerificationStopped state) {
    return VerificationStoppedPage(
      onClosePressed: () => Navigator.pop(context),
      onGiveFeedbackPressed: () => PlaceholderScreen.show(context),
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
      final locale = AppLocalizations.of(context);
      final organizationName = context.read<VerificationBloc>().state.organization?.shortName ?? '-';
      final stopped = await ConfirmActionSheet.show(
        context,
        title: locale.verificationScreenCancelSheetTitle,
        description: locale.verificationScreenCancelSheetDescription(organizationName),
        cancelButtonText: locale.verificationScreenCancelSheetNegativeCta,
        confirmButtonText: locale.verificationScreenCancelSheetPositiveCta,
        confirmButtonColor: Theme.of(context).errorColor,
      );
      if (stopped) bloc.add(VerificationStopRequested(bloc.state.flow));
    } else {
      Navigator.pop(context);
    }
  }
}
