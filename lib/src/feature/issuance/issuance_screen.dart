import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/confirm_action_sheet.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/placeholder_screen.dart';
import '../organization/approve_organization_page.dart';
import 'bloc/issuance_bloc.dart';
import 'page/issuance_card_added_page.dart';
import 'page/issuance_check_data_offering_page.dart';
import 'page/issuance_confirm_pin_page.dart';
import 'page/issuance_generic_error_page.dart';
import 'page/issuance_identity_validation_failed_page.dart';
import 'page/issuance_proof_identity_page.dart';
import 'page/issuance_stopped_page.dart';

class IssuanceScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the IssuanceScreen');
    }
  }

  const IssuanceScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'issuance_scaffold',
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).issuanceScreenTitle),
        leading: _buildBackButton(context),
        actions: [CloseButton(onPressed: () => _stopIssuance(context))],
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<IssuanceBloc>();
          if (bloc.state.canGoBack) {
            bloc.add(const IssuanceBackPressed());
          } else {
            _stopIssuance(context);
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

  Widget _buildStepper() {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      builder: (context, state) {
        Widget? result;
        if (state is IssuanceInitial) result = _buildLoading();
        if (state is IssuanceLoadInProgress) result = _buildLoading();
        if (state is IssuanceCheckOrganization) result = _buildCheckOrganizationPage(context, state);
        if (state is IssuanceProofIdentity) result = _buildProofIdentityPage(context, state);
        if (state is IssuanceProvidePin) result = _buildProvidePinPage(context, state);
        if (state is IssuanceCheckDataOffering) result = _buildCheckDataOfferingPage(context, state);
        if (state is IssuanceCardAdded) result = _buildCardAddedPage(context, state);
        if (state is IssuanceStopped) result = _buildStoppedPage(context, state);
        if (state is IssuanceGenericError) result = _buildGenericErrorPage(context, state);
        if (state is IssuanceIdentityValidationFailure) result = _buildIdentityValidationFailedPage(context, state);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.read<IssuanceBloc>().add(const IssuanceBackPressed()),
        );
      },
    );
  }

  Widget _buildCheckOrganizationPage(BuildContext context, IssuanceCheckOrganization state) {
    return ApproveOrganizationPage(
      onDecline: () => context.read<IssuanceBloc>().add(const IssuanceOrganizationDeclined()),
      onAccept: () => context.read<IssuanceBloc>().add(const IssuanceOrganizationApproved()),
      organization: state.organization,
      purpose: ApprovalPurpose.issuance,
    );
  }

  Widget _buildProofIdentityPage(BuildContext context, IssuanceProofIdentity state) {
    return IssuanceProofIdentityPage(
      onDecline: () => _stopIssuance(context),
      onAccept: () => context.read<IssuanceBloc>().add(const IssuanceShareRequestedAttributesApproved()),
      organization: state.organization,
      attributes: state.requestedAttributes,
    );
  }

  Widget _buildProvidePinPage(BuildContext context, IssuanceProvidePin state) {
    return IssuanceConfirmPinPage(
      onPinValidated: () => context.read<IssuanceBloc>().add(const IssuancePinConfirmed()),
    );
  }

  Widget _buildCheckDataOfferingPage(BuildContext context, IssuanceCheckDataOffering state) {
    return IssuanceCheckDataOfferingPage(
      onDecline: () => _stopIssuance(context),
      onAccept: () => context.read<IssuanceBloc>().add(const IssuanceCheckDataOfferingApproved()),
      attributes: state.flow.cards.first.attributes,
    );
  }

  Widget _buildCardAddedPage(BuildContext context, IssuanceCardAdded state) {
    return IssuanceCardAddedPage(
      onClose: () => _stopIssuance(context),
      cardFront: state.flow.cards.first.front,
    );
  }

  Widget _buildStoppedPage(BuildContext context, IssuanceStopped state) {
    return IssuanceStoppedPage(
      onClosePressed: () => Navigator.pop(context),
      onGiveFeedbackPressed: () => PlaceholderScreen.show(context, 'Give feedback'),
    );
  }

  Widget _buildGenericErrorPage(BuildContext context, IssuanceGenericError state) {
    return IssuanceGenericErrorPage(onClosePressed: () => Navigator.pop(context));
  }

  Widget _buildIdentityValidationFailedPage(BuildContext context, IssuanceIdentityValidationFailure state) {
    return IssuanceIdentityValidationFailedPage(
      onClosePressed: () => Navigator.pop(context),
      onSomethingNotRightPressed: () => PlaceholderScreen.show(context, 'Klopt er iets niet?'),
    );
  }

  void _stopIssuance(BuildContext context) async {
    final bloc = context.read<IssuanceBloc>();
    if (bloc.state.showStopConfirmation) {
      final locale = AppLocalizations.of(context);
      final organizationName = context.read<IssuanceBloc>().state.organization?.shortName ?? '-';
      final stopped = await ConfirmActionSheet.show(
        context,
        title: locale.issuanceStopSheetTitle,
        description: locale.issuanceStopSheetDescription(organizationName),
        cancelButtonText: locale.issuanceStopSheetNegativeCta,
        confirmButtonText: locale.issuanceStopSheetPositiveCta,
      );
      if (stopped) bloc.add(IssuanceStopRequested(bloc.state.flow));
    } else {
      Navigator.pop(context);
    }
  }
}
