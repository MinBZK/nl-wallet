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
        title: Text(AppLocalizations.of(context).signScreenTitle),
        actions: [CloseButton(onPressed: () => _stopSigning(context))],
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
            Expanded(child: _buildPage()),
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

  Widget _buildStepper() {
    return BlocBuilder<SignBloc, SignState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<SignBloc, SignState>(
      builder: (context, state) {
        Widget? result;
        if (state is SignInitial) result = _buildLoading();
        if (state is SignLoadInProgress) result = _buildLoading();
        if (state is SignCheckOrganization) result = _buildCheckOrganization(context, state);
        if (state is SignCheckAgreement) result = _buildCheckAgreement(context, state);
        if (state is SignConfirmAgreement) result = _buildConfirmAgreement(context, state);
        if (state is SignConfirmPin) result = _buildConfirmPin(context, state);
        if (state is SignError) result = _buildError(context);
        if (state is SignStopped) result = _buildStopped(context, state);
        if (state is SignSuccess) result = _buildSuccess(context, state);
        if (result == null) throw UnsupportedError('Unhandled state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
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
    return ApproveOrganizationPage(
      onDecline: () => _stopSigning(context),
      onAccept: () => context.read<SignBloc>().add(const SignOrganizationApproved()),
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
      onDecline: () => _stopSigning(context),
      onAccept: () => context.read<SignBloc>().add(const SignAgreementApproved()),
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
      final locale = AppLocalizations.of(context);
      final stopped = await ConfirmActionSheet.show(
        context,
        title: locale.signScreenCancelSheetTitle,
        description: locale.signScreenCancelSheetDescription,
        cancelButtonText: locale.signScreenCancelSheetNegativeCta,
        confirmButtonText: locale.signScreenCancelSheetPositiveCta,
        confirmButtonColor: Theme.of(context).errorColor,
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
