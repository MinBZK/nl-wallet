import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../../util/extension/build_context_extension.dart';
import 'bloc/pin_bloc.dart';
import 'pin_screen.dart';

/// Cache used as the [initialData] for the stream, this is
/// required to make sure [Hero] animations work as expected.
var _lockedStreamCache = true;

class PinOverlay extends StatelessWidget {
  final Widget child;
  final Stream<bool> isLockedStream;

  @visibleForTesting
  final PinBloc? bloc;

  const PinOverlay({
    required this.child,
    required this.isLockedStream,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final isWalletInitializedUseCase = context.read<IsWalletInitializedUseCase>();
    return StreamBuilder<bool>(
      stream: isLockedStream.asyncMap((locked) async {
        /// Make sure we only lock when the app has an active registration
        return locked && await isWalletInitializedUseCase.invoke();
      }),
      initialData: _lockedStreamCache,
      builder: (context, snapshot) {
        final didChangeState = _lockedStreamCache != snapshot.data!;
        final isLocked = _lockedStreamCache = snapshot.data!;
        if (isLocked) {
          /// Only dismiss when the app was locked this build(), this avoids dismissing new dialogs
          if (didChangeState) {
            _announceLogout(context);
            _dismissOpenDialogs(context);
          }
          return _buildLockedState();
        } else {
          return child;
        }
      },
    );
  }

  Widget _buildLockedState() {
    return BlocProvider<PinBloc>(
      create: (context) => bloc ?? PinBloc(context.read<UnlockWalletWithPinUseCase>()),
      child: PinScreen(
        onUnlock: () {
          /// Unused, locked state is observed above, causing this widget to be replaced
        },
      ),
    );
  }

  void _announceLogout(BuildContext context) {
    SemanticsService.announce(context.l10n.generalWCAGLogoutAnnouncement, TextDirection.ltr);
  }

  void _dismissOpenDialogs(BuildContext context) {
    final navigator = Navigator.of(context);
    Future.microtask(() {
      navigator.popUntil((route) {
        final isDialog = route is ModalBottomSheetRoute || route is DialogRoute;
        return !isDialog;
      });
    });
  }
}
