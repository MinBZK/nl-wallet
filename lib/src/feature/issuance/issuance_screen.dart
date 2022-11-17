import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../organization/approve_organization_page.dart';
import 'bloc/issuance_bloc.dart';
import 'page/check_data_offering_page.dart';
import 'page/issuance_confirm_pin_page.dart';
import 'page/proof_identity_page.dart';

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
        actions: const [CloseButton()],
      ),
      body: Column(
        children: [
          _buildStepper(),
          Expanded(child: _buildPage()),
        ],
      ),
    );
  }

  Widget _buildStepper() {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) {
        return TweenAnimationBuilder<double>(
          builder: (context, progress, child) => LinearProgressIndicator(value: progress),
          duration: kDefaultAnimationDuration,
          tween: Tween<double>(end: state.stepperProgress),
        );
      },
    );
  }

  Widget _buildPage() {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      builder: (context, state) {
        Widget? result;
        if (state is IssuanceInitial) return _buildLoading();
        if (state is IssuanceLoadInProgress) return _buildLoading();
        if (state is IssuanceCheckOrganization) result = _buildCheckOrganizationPage(context, state);
        if (state is IssuanceProofIdentity) result = _buildProofIdentityPage(context, state);
        if (state is IssuanceProvidePin) result = _buildProvidePinPage(context, state);
        if (state is IssuanceCheckDataOffering) return _buildCheckDataOfferingPage(context, state);
        if (state is IssuanceCardAdded) result = Text(state.runtimeType.toString());
        if (state is IssuanceStopped) result = Text(state.runtimeType.toString());
        if (state is IssuanceGenericError) result = Text(state.runtimeType.toString());
        if (state is IssuanceIdentityValidationFailure) result = Text(state.runtimeType.toString());
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return AnimatedSwitcher(duration: kDefaultAnimationDuration, child: result);
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      builder: (context, state) {
        if (state.canGoBack) {
          return BackButton(onPressed: () => context.read<IssuanceBloc>().add(const IssuanceBackPressed()));
        } else {
          return const SizedBox.shrink();
        }
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
    return ProofIdentityPage(
      onDecline: () => context.read<IssuanceBloc>().add(const IssuanceShareRequestedAttributesDeclined()),
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
    return CheckDataOfferingPage(
      onDecline: () => {},
      onAccept: () => {},
      attributes: state.response.cards.first.attributes,
    );
  }
}
