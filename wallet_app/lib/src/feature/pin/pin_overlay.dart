import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
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
    return StreamBuilder<bool>(
      stream: isLockedStream.asyncMap((locked) async {
        /// Make sure we only lock when the app has an active registration
        return locked && await context.read<IsWalletInitializedUseCase>().invoke();
      }),
      initialData: _lockedStreamCache,
      builder: (context, snapshot) {
        final isLocked = _lockedStreamCache = snapshot.data!;
        if (isLocked) {
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
        onUnlock: (_) {
          /// Unused, locked state is observed above, causing this widget to be replaced
        },
      ),
    );
  }
}
