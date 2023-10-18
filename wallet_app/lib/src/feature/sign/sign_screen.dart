import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/sheet/confirm_action_sheet.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../organization/approve/organization_approve_page.dart';
import 'bloc/sign_bloc.dart';
import 'page/check_agreement_page.dart';
import 'page/confirm_agreement_page.dart';
import 'page/sign_confirm_pin_page.dart';
import 'page/sign_generic_error_page.dart';
import 'page/sign_stopped_page.dart';
import 'page/sign_success_page.dart';

class SignScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    try {
      return settings.arguments as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode ${settings.arguments}', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the SignScreen');
    }
  }

  const SignScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: _buildBackButton(context),
        title: Text(context.l10n.signScreenTitle),
        actions: [_buildCloseButton(context)],
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<SignBloc>();
          if (bloc.state.canGoBack) {
            bloc.add(const SignBackPressed());
          } else {
            _stopSigning(context);
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
    return BlocBuilder<SignBloc, SignState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.read<SignBloc>().add(const SignBackPressed()),
        );
      },
    );
  }

  /// The close button stops/closes the sign flow.
  /// It is only visible in the semantics tree when the sign flow is in progress.
  Widget _buildCloseButton(BuildContext context) {
    final closeButton = CloseButton(onPressed: () => _stopSigning(context));
    return BlocBuilder<SignBloc, SignState>(
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
    return BlocBuilder<SignBloc, SignState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<SignBloc, SignState>(
      builder: (context, state) {
        Widget result = switch (state) {
          SignInitial() => _buildLoading(),
          SignLoadInProgress() => _buildLoading(),
          SignCheckOrganization() => _buildCheckOrganization(context, state),
          SignCheckAgreement() => _buildCheckAgreement(context, state),
          SignConfirmAgreement() => _buildConfirmAgreement(context, state),
          SignConfirmPin() => _buildConfirmPin(context, state),
          SignError() => _buildError(context),
          SignStopped() => _buildStopped(context, state),
          SignSuccess() => _buildSuccess(context, state),
        };
        final skipAnim = !state.didGoBack && state is SignCheckOrganization;
        return FakePagingAnimatedSwitcher(
          animateBackwards: state.didGoBack,
          animate: !skipAnim,
          child: result,
        );
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildError(BuildContext context) {
    return SignGenericErrorPage(
      onClosePressed: () => Navigator.pop(context),
    );
  }

  Widget _buildCheckOrganization(BuildContext context, SignCheckOrganization state) {
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopSigning(context),
      onAcceptPressed: () => context.read<SignBloc>().add(const SignOrganizationApproved()),
      organization: state.flow.organization,
      purpose: ApprovalPurpose.sign,
    );
  }

  Widget _buildCheckAgreement(BuildContext context, SignCheckAgreement state) {
    return CheckAgreementPage(
      flow: state.flow,
      onDecline: () => _stopSigning(context),
      onAccept: () => context.read<SignBloc>().add(const SignAgreementChecked()),
    );
  }

  Widget _buildConfirmAgreement(BuildContext context, SignConfirmAgreement state) {
    if (state.flow.hasMissingAttributes) {
      throw UnimplementedError('Not supported, mocks are solely based on data in PID atm.');
    }
    return ConfirmAgreementPage(
      flow: state.flow,
      onDeclinePressed: () => _stopSigning(context),
      onAcceptPressed: () => context.read<SignBloc>().add(const SignAgreementApproved()),
    );
  }

  Widget _buildConfirmPin(BuildContext context, SignConfirmPin state) {
    return SignConfirmPinPage(
      onPinValidated: () => context.read<SignBloc>().add(const SignPinConfirmed()),
    );
  }

  void _stopSigning(BuildContext context) async {
    final bloc = context.read<SignBloc>();
    if (bloc.state.showStopConfirmation) {
      final stopped = await ConfirmActionSheet.show(
        context,
        title: context.l10n.signScreenCancelSheetTitle,
        description: context.l10n.signScreenCancelSheetDescription,
        cancelButtonText: context.l10n.signScreenCancelSheetNegativeCta,
        confirmButtonText: context.l10n.signScreenCancelSheetPositiveCta,
        confirmButtonColor: context.colorScheme.error,
      );
      if (stopped) bloc.add(const SignStopRequested());
    } else {
      Navigator.pop(context);
    }
  }

  Widget _buildStopped(BuildContext context, SignStopped state) {
    return SignStoppedPage(
      onClosePressed: () => Navigator.pop(context),
      onGiveFeedbackPressed: () => PlaceholderScreen.show(context),
    );
  }

  Widget _buildSuccess(BuildContext context, SignSuccess state) {
    return SignSuccessPage(
      organizationName: state.flow.organization.shortName,
      onClosePressed: () => Navigator.pop(context),
      onHistoryPressed: () => Navigator.restorablePushNamed(
        context,
        WalletRoutes.walletHistoryRoute,
      ),
    );
  }
}
