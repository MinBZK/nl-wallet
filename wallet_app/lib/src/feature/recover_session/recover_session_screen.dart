import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/service/navigation_service.dart';
import '../../domain/usecase/session/cancel_session_usecase.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/navigator_state_extension.dart';
import '../common/mixin/lock_state_mixin.dart';
import '../common/page/generic_loading_page.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';

/// Screen displayed when the app is cold started while a disclosure or (PID) issuance session was still active
/// (i.e. the app was killed mid-flow). The splash flow detects this through `WalletStateInDisclosureFlow`/
/// `WalletStateInIssuanceFlow` and routes here instead of resuming the interrupted screen directly, since that
/// screen's bloc state can no longer be reconstructed.
///
/// This screen shows a loading indicator until the wallet is unlocked ([LockStateMixin]), as the wallet is
/// always locked on a cold start. Once unlocked:
/// - If a deeplink was already queued by [NavigationService] (e.g. the user tapped the link that continues
///   the interrupted session), that request is processed, resuming the flow from a fresh screen.
/// - Otherwise the stuck session is cancelled through [CancelSessionUseCase] and the app resets to the splash
///   screen, which re-evaluates the (now no longer 'stuck') [WalletState] and navigates to the correct destination.
///
/// The user can also bail out manually through the cancel/stop action, which triggers the same cancel-and-reset
/// flow without waiting for a queued deeplink (as a fallback mechanism).
class RecoverSessionScreen extends StatefulWidget {
  const RecoverSessionScreen({super.key});

  @override
  State<RecoverSessionScreen> createState() => _RecoverSessionScreenState();
}

class _RecoverSessionScreenState extends State<RecoverSessionScreen> with LockStateMixin {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.recoverSessionScreenTitle),
      ),
      body: SafeArea(
        child: GenericLoadingPage(
          title: context.l10n.recoverSessionScreenTitle,
          description: context.l10n.recoverSessionScreenDescription,
          onCancel: onCancel,
          cancelCta: context.l10n.generalStop,
          loadingIndicator: const SizedBox.shrink(),
        ),
      ),
    );
  }

  Future<void> onCancel() async {
    final context = this.context;
    await context.read<CancelSessionUseCase>().invoke();
    if (context.mounted) await Navigator.of(context).resetToSplash();
  }

  @override
  FutureOr<void> onLock() {}

  @override
  Future<void> onUnlock() async {
    final navigationService = context.read<NavigationService>();
    if (navigationService.hasQueuedRequest) {
      unawaited(navigationService.processQueue());
    } else {
      unawaited(onCancel());
    }
  }
}
