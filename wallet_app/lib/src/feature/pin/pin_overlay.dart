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
      stream: isLockedStream,
      initialData: _lockedStreamCache,
      builder: (context, snapshot) {
        final isLocked = _lockedStreamCache = snapshot.data!;
        if (isLocked) {
          /// Check for initialization to make sure we don't show the
          /// pin overlay when the app is not initialized yet/anymore.
          return FutureBuilder(
            future: context.read<IsWalletInitializedUseCase>().invoke(),
            builder: (context, isInitialized) {
              final loading = !isInitialized.hasData && !isInitialized.hasError;
              if (loading) return child;
              if (isInitialized.hasError) return _buildLockedState();
              if (isInitialized.data!) return _buildLockedState();
              return child;
            },
          );
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
