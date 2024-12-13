import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../data/service/navigation_service.dart';
import '../../domain/model/update/update_notification.dart';
import '../../domain/usecase/update/observe_version_state_usecase.dart';
import '../common/widget/minimal_wallet_app.dart';
import 'app_blocked_screen.dart';

/// This widget observes and processes the update state of the app
/// It intentionally lives above the [WalletApp] widget to make sure
/// no accidental navigation is possible if the app enters the [VersionStateBlock]
/// state. This also means that the [BuildContext] in this widget does not have
/// access to the [Theme] or l10n, which is something to consider while working
/// on this widget (and e.g. the reason we show dialogs through [NavigationService]).
class UpdateChecker extends StatefulWidget {
  final Widget child;

  const UpdateChecker({
    required this.child,
    super.key,
  });

  @override
  State<UpdateChecker> createState() => _UpdateCheckerState();
}

class _UpdateCheckerState extends State<UpdateChecker> {
  // Whether the device is in a blocked state
  final ValueNotifier<bool> _isBlocked = ValueNotifier(false);
  StreamSubscription? observeVersionSubscription;

  NavigationService get navigationService => context.read();

  @override
  void initState() {
    super.initState();

    /// Note that the stream contains a debounce to make sure the Navigator has a chance to settle. When this is omitted the app can
    /// crash when the initial state triggers a dialog (e.g. [VersionRecommend]) and navigation is requested before
    /// the Navigator is initialized properly (i.e. attached to the [WalletApp] that lives below this widget).
    observeVersionSubscription = context.read<ObserveVersionStateUsecase>().invoke().listen(_onVersionStateUpdated);
  }

  Future<void> _onVersionStateUpdated(VersionState state) async {
    switch (state) {
      case VersionStateOk():
      case VersionStateNotify():
        return;
      case VersionStateRecommend():
        await navigationService.processUpdateNotification(RecommendUpdateNotification());
        return;
      case VersionStateWarn():
        await navigationService.processUpdateNotification(
          WarnUpdateNotification(timeUntilBlocked: state.timeUntilBlocked),
        );
        return;
      case VersionStateBlock():
        _isBlocked.value = true;
        return;
    }
  }

  @override
  void dispose() {
    observeVersionSubscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder(
      valueListenable: _isBlocked,
      builder: (c, isBlocked, child) {
        if (isBlocked) return const MinimalWalletApp(child: AppBlockedScreen());
        return child!;
      },
      child: widget.child,
    );
  }
}
