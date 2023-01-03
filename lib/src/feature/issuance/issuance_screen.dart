import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/wallet_card.dart';
import '../../wallet_routes.dart';
import '../common/widget/animated_linear_progress_indicator.dart';
import '../common/widget/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/confirm_action_sheet.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/placeholder_screen.dart';
import '../data_incorrect/data_incorrect_screen.dart';
import '../organization/approve_organization_page.dart';
import 'argument/issuance_screen_argument.dart';
import 'bloc/issuance_bloc.dart';
import 'page/issuance_check_card_page.dart';
import 'page/issuance_check_data_offering_page.dart';
import 'page/issuance_confirm_pin_page.dart';
import 'page/issuance_generic_error_page.dart';
import 'page/issuance_identity_validation_failed_page.dart';
import 'page/issuance_proof_identity_page.dart';
import 'page/issuance_select_cards_page.dart';
import 'page/issuance_stopped_page.dart';
import 'page/issuance_success_page.dart';

class IssuanceScreen extends StatelessWidget {
  static IssuanceScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return IssuanceScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [IssuanceScreenArgument] when opening the IssuanceScreen');
    }
  }

  const IssuanceScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'issuance_scaffold',
      appBar: AppBar(
        title: _buildTitle(context),
        leading: _buildBackButton(context),
        actions: [CloseButton(onPressed: () => _stopIssuance(context))],
      ),
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.bloc;
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

  Widget _buildTitle(BuildContext context) {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      buildWhen: (previous, current) => current is IssuanceInitial && current is IssuanceLoadInProgress,
      builder: (context, state) {
        final locale = AppLocalizations.of(context);
        if (state.isRefreshFlow) {
          return Text(locale.issuanceScreenRefreshTitle);
        } else {
          return Text(locale.issuanceScreenTitle);
        }
      },
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
        if (state is IssuanceSelectCards) result = _buildSelectCardsPage(context, state);
        if (state is IssuanceCheckCards) result = _buildCheckCardsPage(context, state);
        if (state is IssuanceCompleted) result = _buildIssuanceCompletedPage(context, state);
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
          onPressed: () => context.bloc.add(const IssuanceBackPressed()),
        );
      },
    );
  }

  Widget _buildCheckOrganizationPage(BuildContext context, IssuanceCheckOrganization state) {
    return ApproveOrganizationPage(
      onDecline: () => _stopIssuance(context),
      onAccept: () => context.bloc.add(const IssuanceOrganizationApproved()),
      organization: state.organization,
      purpose: ApprovalPurpose.issuance,
    );
  }

  Widget _buildProofIdentityPage(BuildContext context, IssuanceProofIdentity state) {
    return IssuanceProofIdentityPage(
      onDecline: () => _stopIssuance(context),
      onAccept: () => context.bloc.add(const IssuanceShareRequestedAttributesApproved()),
      flow: state.flow,
      isRefreshFlow: state.isRefreshFlow,
    );
  }

  Widget _buildProvidePinPage(BuildContext context, IssuanceProvidePin state) {
    return IssuanceConfirmPinPage(
      onPinValidated: () => context.bloc.add(const IssuancePinConfirmed()),
    );
  }

  Widget _buildCheckDataOfferingPage(BuildContext context, IssuanceCheckDataOffering state) {
    return IssuanceCheckDataOfferingPage(
      onDecline: () async {
        final result = await DataIncorrectScreen.show(context);
        if (result == null) return;
        switch (result) {
          case DataIncorrectResult.declineCard:
            context.bloc.add(IssuanceStopRequested(state.flow));
            break;
          case DataIncorrectResult.acceptCard:
            context.bloc.add(const IssuanceCheckDataOfferingApproved());
            break;
        }
      },
      onAccept: () => context.bloc.add(const IssuanceCheckDataOfferingApproved()),
      attributes: state.flow.cards.first.attributes,
    );
  }

  Widget _buildIssuanceCompletedPage(BuildContext context, IssuanceCompleted state) {
    return IssuanceSuccessPage(
      onClose: () => Navigator.restorablePushNamedAndRemoveUntil(
        context,
        WalletRoutes.homeRoute,
        ModalRoute.withName(WalletRoutes.splashRoute),
      ),
      cards: state.addedCards.map((e) => e.front).toList(),
      isRefreshFlow: state.isRefreshFlow,
    );
  }

  Widget _buildStoppedPage(BuildContext context, IssuanceStopped state) {
    return IssuanceStoppedPage(
      onClosePressed: () => Navigator.pop(context),
      onGiveFeedbackPressed: () => PlaceholderScreen.show(context),
    );
  }

  Widget _buildGenericErrorPage(BuildContext context, IssuanceGenericError state) {
    return IssuanceGenericErrorPage(onClosePressed: () => Navigator.pop(context));
  }

  Widget _buildIdentityValidationFailedPage(BuildContext context, IssuanceIdentityValidationFailure state) {
    return IssuanceIdentityValidationFailedPage(
      onClosePressed: () => Navigator.pop(context),
      onSomethingNotRightPressed: () => PlaceholderScreen.show(context),
    );
  }

  void _stopIssuance(BuildContext context) async {
    final bloc = context.bloc;
    if (bloc.state.showStopConfirmation) {
      final locale = AppLocalizations.of(context);
      final organizationName = bloc.state.organization?.shortName ?? '-';
      final stopped = await ConfirmActionSheet.show(
        context,
        title: locale.issuanceStopSheetTitle,
        description: locale.issuanceStopSheetDescription(organizationName),
        cancelButtonText: locale.issuanceStopSheetNegativeCta,
        confirmButtonText: locale.issuanceStopSheetPositiveCta,
        confirmButtonColor: Theme.of(context).errorColor,
      );
      if (stopped) bloc.add(IssuanceStopRequested(bloc.state.flow));
    } else {
      Navigator.pop(context);
    }
  }

  Widget? _buildSelectCardsPage(BuildContext context, IssuanceSelectCards state) {
    return IssuanceSelectCardsPage(
      cards: state.availableCards,
      selectedCardIds: state.multipleCardsFlow.selectedCardIds.toList(),
      onCardSelectionToggled: (WalletCard card) => context.bloc.add(IssuanceCardToggled(card)),
      onAddSelectedPressed: () => context.bloc.add(const IssuanceSelectedCardsConfirmed()),
      onStopPressed: () => _stopIssuance(context),
      showNoSelectionError: state.showNoSelectionError,
    );
  }

  Widget _buildCheckCardsPage(BuildContext context, IssuanceCheckCards state) {
    return IssuanceCheckCardPage(
      key: ValueKey(state.cardToCheck.id),
      card: state.cardToCheck,
      onAccept: () => context.bloc.add(IssuanceCardApproved(state.cardToCheck)),
      onDecline: () async {
        final result = await DataIncorrectScreen.show(context);
        if (result == null) return; //Screen dismissed without explicit action.
        switch (result) {
          case DataIncorrectResult.declineCard:
            context.bloc.add(IssuanceCardDeclined(state.cardToCheck));
            break;
          case DataIncorrectResult.acceptCard:
            context.bloc.add(IssuanceCardApproved(state.cardToCheck));
            break;
        }
      },
      totalNrOfCards: state.totalNrOfCardsToCheck,
      currentCardIndex: state.multipleCardsFlow.activeIndex,
    );
  }
}

extension _IssuanceScreenExtensions on BuildContext {
  IssuanceBloc get bloc => read<IssuanceBloc>();
}
