import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/wallet_card.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/sheet/confirm_action_sheet.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/wallet_app_bar.dart';
import '../dashboard/dashboard_screen.dart';
import '../data_incorrect/data_incorrect_screen.dart';
import '../organization/approve/organization_approve_page.dart';
import '../organization/detail/organization_detail_screen.dart';
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
        title: _buildTitle(context),
        leading: _buildBackButton(context),
        automaticallyImplyLeading: false,
        actions: [CloseIconButton(onPressed: () => _stopIssuance(context))],
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
            bloc.add(const IssuanceBackPressed());
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

  Widget _buildTitle(BuildContext context) {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      buildWhen: (previous, current) => current is IssuanceInitial && current is IssuanceLoadInProgress,
      builder: (context, state) {
        if (state.isRefreshFlow) {
          return Text(context.l10n.issuanceScreenRefreshTitle);
        } else {
          return Text(context.l10n.issuanceScreenTitle);
        }
      },
    );
  }

  Widget _buildPage() {
    return BlocBuilder<IssuanceBloc, IssuanceState>(
      builder: (context, state) {
        final Widget result = switch (state) {
          IssuanceInitial() => _buildLoading(),
          IssuanceLoadInProgress() => _buildLoading(),
          IssuanceCheckOrganization() => _buildCheckOrganizationPage(context, state),
          IssuanceMissingAttributes() => _buildMissingAttributes(context, state),
          IssuanceProofIdentity() => _buildProofIdentityPage(context, state),
          IssuanceProvidePin() => _buildProvidePinPage(context, state),
          IssuanceCheckDataOffering() => _buildCheckDataOfferingPage(context, state),
          IssuanceSelectCards() => _buildSelectCardsPage(context, state),
          IssuanceCheckCards() => _buildCheckCardsPage(context, state),
          IssuanceCompleted() => _buildIssuanceCompletedPage(context, state),
          IssuanceStopped() => _buildStoppedPage(context, state),
          IssuanceGenericError() => _buildGenericErrorPage(context),
          IssuanceIdentityValidationFailure() => _buildIdentityValidationFailedPage(context, state),
          IssuanceLoadFailure() => _buildGenericErrorPage(context),
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

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget? _buildBackButton(BuildContext context) {
    final canGoBack = context.watch<IssuanceBloc>().state.canGoBack;
    if (!canGoBack) return null;
    return BackIconButton(
      onPressed: () => context.bloc.add(const IssuanceBackPressed()),
    );
  }

  Widget _buildCheckOrganizationPage(BuildContext context, IssuanceCheckOrganization state) {
    return OrganizationApprovePage(
      onDeclinePressed: () => _stopIssuance(context),
      originUrl: 'http://issue.origin.org',
      onAcceptPressed: () => context.bloc.add(const IssuanceOrganizationApproved()),
      organization: state.organization,
      purpose: ApprovalPurpose.issuance,
      onShowDetailsPressed: () {
        OrganizationDetailScreen.showPreloaded(
          context,
          state.organization,
          sharedDataWithOrganizationBefore: false,
        );
      },
    );
  }

  Widget _buildMissingAttributes(BuildContext context, IssuanceMissingAttributes state) {
    throw UnimplementedError('This screen is not implemented since all mock issuance flows only rely on PID');
  }

  Widget _buildProofIdentityPage(BuildContext context, IssuanceProofIdentity state) {
    return IssuanceProofIdentityPage(
      onDeclinePressed: () => _stopIssuance(context),
      onAcceptPressed: () => context.bloc.add(const IssuanceShareRequestedAttributesApproved()),
      organization: state.organization,
      attributes: state.requestedAttributes,
      policy: state.policy,
      isRefreshFlow: state.isRefreshFlow,
    );
  }

  Widget _buildProvidePinPage(BuildContext context, IssuanceProvidePin state) {
    return IssuanceConfirmPinPage(
      onPinValidated: (_) => context.bloc.add(const IssuancePinConfirmed()),
    );
  }

  Widget _buildCheckDataOfferingPage(BuildContext context, IssuanceCheckDataOffering state) {
    return IssuanceCheckDataOfferingPage(
      onDeclinePressed: () async {
        final bloc = context.bloc;
        final result = await DataIncorrectScreen.show(context);
        if (result == null) return;
        switch (result) {
          case DataIncorrectResult.declineCard:
            bloc.add(const IssuanceStopRequested());
          case DataIncorrectResult.acceptCard:
            bloc.add(const IssuanceCheckDataOfferingApproved());
        }
      },
      onAcceptPressed: () => context.bloc.add(const IssuanceCheckDataOfferingApproved()),
      attributes: state.card.attributes,
    );
  }

  Widget _buildIssuanceCompletedPage(BuildContext context, IssuanceCompleted state) {
    return IssuanceSuccessPage(
      onClose: () => DashboardScreen.show(context),
      cards: state.addedCards,
      isRefreshFlow: state.isRefreshFlow,
    );
  }

  Widget _buildStoppedPage(BuildContext context, IssuanceStopped state) {
    return IssuanceStoppedPage(
      onClosePressed: () => Navigator.pop(context),
      onGiveFeedbackPressed: () => PlaceholderScreen.showGeneric(context),
    );
  }

  Widget _buildGenericErrorPage(BuildContext context) {
    return IssuanceGenericErrorPage(onClosePressed: () => Navigator.pop(context));
  }

  Widget _buildIdentityValidationFailedPage(BuildContext context, IssuanceIdentityValidationFailure state) {
    return IssuanceIdentityValidationFailedPage(
      onClosePressed: () => Navigator.pop(context),
      onSomethingNotRightPressed: () => PlaceholderScreen.showGeneric(context),
    );
  }

  Future<void> _stopIssuance(BuildContext context) async {
    final bloc = context.bloc;
    if (bloc.state.showStopConfirmation) {
      final organizationName = bloc.organization?.displayName ?? '-'.untranslated;
      final stopped = await ConfirmActionSheet.show(
        context,
        title: context.l10n.issuanceStopSheetTitle,
        description: context.l10n.issuanceStopSheetDescription(organizationName.l10nValue(context)),
        cancelButtonText: context.l10n.issuanceStopSheetNegativeCta,
        confirmButtonText: context.l10n.issuanceStopSheetPositiveCta,
        confirmButtonColor: context.colorScheme.error,
      );
      if (stopped) bloc.add(const IssuanceStopRequested());
    } else {
      Navigator.pop(context);
    }
  }

  Widget _buildSelectCardsPage(BuildContext context, IssuanceSelectCards state) {
    return IssuanceSelectCardsPage(
      cards: state.cards,
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
      onAcceptPressed: () => context.bloc.add(IssuanceCardApproved(state.cardToCheck)),
      onDeclinePressed: () async {
        final bloc = context.bloc;
        final result = await DataIncorrectScreen.show(context);
        if (result == null) return; //Screen dismissed without explicit action.
        switch (result) {
          case DataIncorrectResult.declineCard:
            bloc.add(IssuanceCardDeclined(state.cardToCheck));
          case DataIncorrectResult.acceptCard:
            bloc.add(IssuanceCardApproved(state.cardToCheck));
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
