import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/cast_util.dart';
import '../../util/extension/biometrics_extension.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/helper/announcements_helper.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/page/generic_loading_page.dart';
import '../common/page/terminal_page.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/button/icon/info_icon_button.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import '../error/error_screen.dart';
import '../pin_dialog/pin_confirmation_error_dialog.dart';
import '../pin_dialog/pin_validation_error_dialog.dart';
import 'bloc/setup_security_bloc.dart';
import 'page/setup_security_completed_page.dart';
import 'page/setup_security_pin_page.dart';

const _kSelectPinScreenKey = ValueKey('selectPinScreen');
const _kConfirmPinScreenKey = ValueKey('confirmPinScreen');

class SetupSecurityScreen extends StatelessWidget {
  const SetupSecurityScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watch<SetupSecurityBloc>().state;
    return ScrollOffsetProvider(
      debugLabel: 'setup_security',
      child: Scaffold(
        restorationId: 'setup_security_scaffold',
        appBar: WalletAppBar(
          automaticallyImplyLeading: false,
          leading: _buildBackButton(context, state),
          actions: [_buildAboutAction(context, state)],
          title: _buildTitle(context, state),
          progress: state.stepperProgress,
        ),
        body: PopScope(
          canPop: state is SetupSecuritySelectPinInProgress,
          onPopInvokedWithResult: (didPop, result) {
            if (!didPop) context.bloc.add(SetupSecurityBackPressed());
          },
          child: SafeArea(
            child: _buildPage(),
          ),
        ),
      ),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<SetupSecurityBloc, SetupSecurityState>(
      listener: (context, state) async {
        unawaited(_runAnnouncements(context, state));
        final bloc = context.bloc;
        switch (state) {
          case SetupSecurityGenericError():
            ErrorScreen.showGeneric(context, secured: false, style: ErrorCtaStyle.retry);
          case SetupSecurityNetworkError():
            ErrorScreen.showNetwork(context, networkError: tryCast(state), secured: false);
          case SetupSecuritySelectPinFailed():
            await PinValidationErrorDialog.show(context, state.reason).then((_) => bloc.add(PinBackspacePressed()));
          case SetupSecurityPinConfirmationFailed():
            await PinConfirmationErrorDialog.show(context, retryAllowed: state.retryAllowed).then((_) {
              bloc.add(state.retryAllowed ? PinBackspacePressed() : SetupSecurityRetryPressed());
            });
          case SetupSecurityDeviceIncompatibleError():
            ErrorScreen.showDeviceIncompatible(context);
          default:
            break;
        }
      },
      builder: (context, state) {
        final Widget result = switch (state) {
          SetupSecuritySelectPinInProgress() => _buildSelectPinPage(context, enteredDigits: state.enteredDigits),
          SetupSecuritySelectPinFailed() => _buildSelectPinPage(context, enteredDigits: kPinDigits),
          SetupSecurityPinConfirmationInProgress() =>
            _buildPinConfirmationPage(context, enteredDigits: state.enteredDigits),
          SetupSecurityPinConfirmationFailed() => _buildPinConfirmationPage(context, enteredDigits: kPinDigits),
          SetupSecurityCreatingWallet() => _buildCreatingWallet(context, state),
          SetupSecurityCompleted() => _buildSetupCompletedPage(context, state),
          SetupSecurityGenericError() => _buildSetupFailed(context),
          SetupSecurityNetworkError() => _buildSetupFailed(context),
          SetupSecurityDeviceIncompatibleError() => _buildSetupFailed(context),
          SetupSecurityConfigureBiometrics() => _buildConfigureBiometricsPage(context, state),
        };
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Future<void> _runAnnouncements(BuildContext context, SetupSecurityState state) async {
    if (!context.isScreenReaderEnabled) return;
    final l10n = context.l10n;
    await Future.delayed(kDefaultAnnouncementDelay);

    if (state is SetupSecuritySelectPinInProgress) {
      if (state.afterBackspacePressed) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      }
    }
    if (state is SetupSecurityPinConfirmationInProgress) {
      if (state.afterBackspacePressed) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      } else if (state.enteredDigits == 0) {
        await SemanticsService.announce(l10n.setupSecurityScreenWCAGPinChosenAnnouncement, TextDirection.ltr);
      } else if (state.enteredDigits > 0 && state.enteredDigits < kPinDigits) {
        AnnouncementsHelper.announceEnteredDigits(l10n, state.enteredDigits);
      }
    }
  }

  Widget? _buildBackButton(BuildContext context, SetupSecurityState state) {
    if (!state.canGoBack) return null; // Allow title to align to the left in [WalletAppBar].
    return AnimatedVisibilityBackButton(
      visible: state.canGoBack,
      onPressed: () {
        if (state is SetupSecuritySelectPinInProgress) {
          Navigator.maybePop(context);
        } else {
          context.bloc.add(SetupSecurityBackPressed());
        }
      },
    );
  }

  Widget _buildAboutAction(BuildContext context, SetupSecurityState state) {
    if (state is SetupSecurityCompleted) return const SizedBox.shrink();
    return const InfoIconButton();
  }

  Widget _buildSelectPinPage(BuildContext context, {required int enteredDigits}) {
    return SetupSecurityPinPage(
      key: _kSelectPinScreenKey,
      title: context.l10n.setupSecuritySelectPinPageTitle,
      enteredDigits: enteredDigits,
      onKeyPressed: (digit) => context.bloc.add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.bloc.add(PinBackspacePressed()),
      onBackspaceLongPressed: () => context.bloc.add(PinClearPressed()),
    );
  }

  Widget _buildPinConfirmationPage(BuildContext context, {required int enteredDigits}) {
    return SetupSecurityPinPage(
      key: _kConfirmPinScreenKey,
      title: context.l10n.setupSecurityConfirmationPageTitle,
      enteredDigits: enteredDigits,
      onKeyPressed: (digit) => context.bloc.add(PinDigitPressed(digit)),
      onBackspacePressed: () => context.bloc.add(PinBackspacePressed()),
      onBackspaceLongPressed: () => context.bloc.add(PinClearPressed()),
    );
  }

  Widget _buildCreatingWallet(BuildContext context, SetupSecurityCreatingWallet state) {
    return GenericLoadingPage(
      title: context.l10n.setupSecurityLoadingPageTitle,
      description: context.l10n.setupSecurityLoadingPageDescription,
    );
  }

  Widget _buildConfigureBiometricsPage(BuildContext context, SetupSecurityConfigureBiometrics state) {
    final String title = context.l10n.setupBiometricsPageTitle(state.biometrics.prettyPrint(context));
    final String illustration = switch (state.biometrics) {
      Biometrics.face => WalletAssets.svg_biometrics_face,
      Biometrics.fingerprint => WalletAssets.svg_biometrics_finger,
      Biometrics.some => WalletAssets.svg_biometrics_finger,
      Biometrics.none => throw UnsupportedError('Biometrics cant be configured when none are available'),
    };
    return TerminalPage(
      title: title,
      description: context.l10n.setupBiometricsPageDescription,
      primaryButtonCta: context.l10n.setupBiometricsPageEnableCta,
      illustration: PageIllustration(asset: illustration),
      onPrimaryPressed: () => context.bloc.add(EnableBiometricsPressed()),
      secondaryButtonCta: context.l10n.setupBiometricsPageSkipCta,
      primaryButtonIcon: Icon(state.biometrics.icon),
      onSecondaryButtonPressed: () => context.bloc.add(SkipBiometricsPressed()),
    );
  }

  Widget _buildSetupCompletedPage(BuildContext context, SetupSecurityCompleted state) {
    return SetupSecurityCompletedPage(
      key: const Key('setupSecurityCompletedPage'),
      enabledBiometrics: state.enabledBiometrics,
      onSetupWalletPressed: () => Navigator.of(context).restorablePushNamedAndRemoveUntil(
        WalletRoutes.walletPersonalizeRoute,
        ModalRoute.withName(WalletRoutes.splashRoute),
      ),
    );
  }

  /// This is more a placeholder/fallback over anything else.
  /// Whenever the user is hit with a [SetupSecurityGenericError] or [SetupSecurityNetworkError]
  /// this is built, but the listener should trigger the [ErrorScreen] while the bloc resets
  /// the flow so the user can try again. That said, to be complete we need to build something
  /// in this state, hence this method is kept around.
  Widget _buildSetupFailed(BuildContext context) {
    return ErrorPage.generic(
      context,
      style: ErrorCtaStyle.retry,
      onPrimaryActionPressed: () => context.bloc.add(SetupSecurityRetryPressed()),
    );
  }

  Widget _buildTitle(BuildContext context, SetupSecurityState state) {
    final String title = switch (state) {
      SetupSecuritySelectPinInProgress() => '',
      SetupSecuritySelectPinFailed() => '',
      SetupSecurityPinConfirmationInProgress() => '',
      SetupSecurityPinConfirmationFailed() => '',
      SetupSecurityCreatingWallet() => '',
      SetupSecurityCompleted() => context.l10n.setupSecurityCompletedPageTitle,
      SetupSecurityGenericError() => '',
      SetupSecurityNetworkError() => '',
      SetupSecurityDeviceIncompatibleError() => '',
      SetupSecurityConfigureBiometrics() => context.l10n.setupBiometricsPageTitle(
          state.biometrics.prettyPrint(context),
        ),
    };
    if (title.isEmpty) return const SizedBox.shrink();
    return TitleText(title);
  }
}

extension _SetupSecurityScreenExtension on BuildContext {
  SetupSecurityBloc get bloc => read<SetupSecurityBloc>();
}
