import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/flow_progress.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/helper/announcements_helper.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/page/generic_loading_page.dart';
import '../common/page/terminal_page.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import '../error/error_screen.dart';
import '../pin_dialog/pin_confirmation_error_dialog.dart';
import '../pin_dialog/pin_validation_error_dialog.dart';
import 'bloc/change_pin_bloc.dart';
import 'page/enter_current_pin_page.dart';
import 'page/select_new_pin_page.dart';

const kTotalNrOfPages = 4;
const _kSelectNewPinPageKey = ValueKey('select_new_pin_page');
const _kConfirmNewPinPageKey = ValueKey('confirm_new_pin_page');

class ChangePinScreen extends StatelessWidget {
  @visibleForTesting
  final bool forceAnnouncements;

  const ChangePinScreen({this.forceAnnouncements = false, super.key});

  @override
  Widget build(BuildContext context) {
    return ScrollOffsetProvider(
      child: Scaffold(
        appBar: WalletAppBar(
          title: _buildTitle(context),
          progress: _buildFlowProgress(context),
          leading: _buildLeading(context),
        ),
        key: const Key('changePinScreen'),
        body: SafeArea(
          child: BlocConsumer<ChangePinBloc, ChangePinState>(
            listener: (BuildContext context, ChangePinState state) async {
              final bloc = context.bloc;
              unawaited(_runAnnouncements(context, state));
              switch (state) {
                case ChangePinGenericError():
                  ErrorScreen.showGeneric(context, secured: false, style: ErrorCtaStyle.retry);
                case ChangePinNetworkError():
                  ErrorScreen.showNetwork(context, networkError: tryCast(state), secured: false);
                case ChangePinSelectNewPinFailed():
                  await PinValidationErrorDialog.show(context, state.reason)
                      .then((_) => bloc.add(PinBackspacePressed()));
                case ChangePinConfirmNewPinFailed():
                  await PinConfirmationErrorDialog.show(context, retryAllowed: state.retryAllowed).then((_) {
                    bloc.add(state.retryAllowed ? PinBackspacePressed() : ChangePinRetryPressed());
                  });
                default:
                  break;
              }
            },
            builder: (context, state) {
              final Widget result = switch (state) {
                ChangePinInitial() => _buildEnterCurrentPinPage(context),
                ChangePinSelectNewPinInProgress() =>
                  _buildSelectNewPinPage(context, enteredDigits: state.enteredDigits),
                ChangePinSelectNewPinFailed() => _buildSelectNewPinPage(context, enteredDigits: kPinDigits),
                ChangePinConfirmNewPinInProgress() =>
                  _buildConfirmNewPinPage(context, enteredDigits: state.enteredDigits),
                ChangePinConfirmNewPinFailed() => _buildConfirmNewPinPage(context, enteredDigits: kPinDigits),
                ChangePinUpdating() => _buildChangePinUpdating(context),
                ChangePinCompleted() => _buildChangePinSuccess(context),
                ChangePinGenericError() => _buildChangePinFailed(context),
                ChangePinNetworkError() => _buildChangePinFailed(context),
              };
              return FakePagingAnimatedSwitcher(
                animateBackwards: state.didGoBack,
                child: result,
              );
            },
          ),
        ),
      ),
    );
  }

  FlowProgress _buildFlowProgress(BuildContext context) {
    final state = context.watch<ChangePinBloc>().state;

    final currentStep = switch (state) {
      ChangePinInitial() => 1,
      ChangePinSelectNewPinInProgress() => 2,
      ChangePinSelectNewPinFailed() => 2,
      ChangePinConfirmNewPinInProgress() => 3,
      ChangePinConfirmNewPinFailed() => 3,
      ChangePinUpdating() => 3,
      ChangePinCompleted() => kTotalNrOfPages,
      ChangePinGenericError() => 0,
      ChangePinNetworkError() => 0,
    };

    return FlowProgress(currentStep: currentStep, totalSteps: kTotalNrOfPages);
  }

  Widget _buildLeading(BuildContext context) {
    return BlocBuilder<ChangePinBloc, ChangePinState>(
      builder: (context, state) {
        final isUpdatingOrCompleted = state is ChangePinUpdating || state is ChangePinCompleted;
        return AnimatedVisibilityBackButton(
          visible: !isUpdatingOrCompleted,
          onPressed: () {
            final isInitialOrCompleted = state is ChangePinInitial || state is ChangePinCompleted;
            if (isInitialOrCompleted) {
              Navigator.pop(context);
            } else {
              context.bloc.add(ChangePinBackPressed());
            }
          },
        );
      },
    );
  }

  Widget _buildEnterCurrentPinPage(BuildContext context) {
    return EnterCurrentPinPage(
      onPinValidated: (pin) => context.bloc.add(ChangePinCurrentPinValidated(pin)),
    );
  }

  Widget _buildSelectNewPinPage(BuildContext context, {required int enteredDigits}) {
    return SelectNewPinPage(
      key: _kSelectNewPinPageKey,
      title: context.l10n.changePinScreenSelectNewPinTitle,
      enteredDigits: enteredDigits,
      onKeyPressed: (digit) => context.bloc.add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.bloc.add(PinBackspacePressed()),
      onBackspaceLongPressed: () => context.bloc.add(PinClearPressed()),
    );
  }

  Widget _buildConfirmNewPinPage(BuildContext context, {required int enteredDigits}) {
    return SelectNewPinPage(
      key: _kConfirmNewPinPageKey,
      title: context.l10n.changePinScreenConfirmNewPinTitle,
      enteredDigits: enteredDigits,
      onKeyPressed: (digit) => context.bloc.add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.bloc.add(PinBackspacePressed()),
      onBackspaceLongPressed: () => context.bloc.add(PinClearPressed()),
    );
  }

  /// This is more a placeholder/fallback over anything else.
  /// Whenever the user is hit with a [ChangePinGenericError] or [ChangePinNetworkError]
  /// this is built, but the listener should trigger the [ErrorScreen] while the bloc resets
  /// the flow so the user can try again. That said, to be complete we need to build something
  /// in this state, hence this method is kept around.
  Widget _buildChangePinFailed(BuildContext context) {
    return ErrorPage.generic(
      context,
      style: ErrorCtaStyle.retry,
      onPrimaryActionPressed: () => context.bloc.add(ChangePinRetryPressed()),
    );
  }

  Widget _buildChangePinUpdating(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.changePinScreenUpdatingTitle,
      description: context.l10n.changePinScreenUpdatingDescription,
    );
  }

  Widget _buildChangePinSuccess(BuildContext context) {
    return TerminalPage(
      title: context.l10n.changePinScreenSuccessTitle,
      description: context.l10n.changePinScreenSuccessDescription,
      primaryButtonCta: context.l10n.changePinScreenToOverviewCta,
      secondaryButtonCta: context.l10n.changePinScreenToSettingsCta,
      illustration: const PageIllustration(asset: WalletAssets.svg_pin_set),
      onPrimaryPressed: () => Navigator.pushNamedAndRemoveUntil(
        context,
        WalletRoutes.dashboardRoute,
        ModalRoute.withName(WalletRoutes.splashRoute),
      ),
      onSecondaryButtonPressed: () => Navigator.pop(context),
    );
  }

  Future<void> _runAnnouncements(BuildContext context, ChangePinState state) async {
    if (!context.isScreenReaderEnabled && !forceAnnouncements) return;
    final l10n = context.l10n;
    await Future.delayed(kDefaultAnnouncementDelay);

    if (state is ChangePinSelectNewPinInProgress) {
      if (state.afterBackspacePressed) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      }
    }
    if (state is ChangePinConfirmNewPinInProgress) {
      if (state.afterBackspacePressed) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      } else if (state.enteredDigits == 0) {
        await SemanticsService.announce(l10n.setupSecurityScreenWCAGPinChosenAnnouncement, TextDirection.ltr);
      } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      }
    }
  }

  Widget _buildTitle(BuildContext context) {
    final state = context.watch<ChangePinBloc>().state;
    switch (state) {
      case ChangePinCompleted():
        return TitleText(context.l10n.changePinScreenSuccessTitle);
      case ChangePinInitial():
      case ChangePinSelectNewPinInProgress():
      case ChangePinSelectNewPinFailed():
      case ChangePinConfirmNewPinInProgress():
      case ChangePinConfirmNewPinFailed():
      case ChangePinUpdating():
      case ChangePinGenericError():
      case ChangePinNetworkError():
        return const TitleText('');
    }
  }
}

extension _ChangePinScreenExtensions on BuildContext {
  ChangePinBloc get bloc => read<ChangePinBloc>();
}
