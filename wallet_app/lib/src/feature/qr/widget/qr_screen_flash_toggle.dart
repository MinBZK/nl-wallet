import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/flashlight_cubit.dart';

/// Explicit naming because it relies on the [DefaultTabController]
/// and [FlashlightCubit] and assumes the button should only be shown
/// on the second tab. As such, only really relevant as a child
/// of the QrScreen.
class QrScreenFlashToggle extends StatelessWidget {
  const QrScreenFlashToggle({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FadeTransition(
      opacity: ReverseAnimation(DefaultTabController.of(context).animation!),
      child: BlocBuilder<FlashlightCubit, FlashlightState>(
        builder: (context, state) {
          return switch (state) {
            FlashlightInitial() => const SizedBox.shrink(),
            FlashlightToggled() => const IconButton(
                icon: Icon(Icons.flashlight_on),
                onPressed: null,
              ),
            FlashlightSuccess() => IconButton(
                icon: Icon(state.on ? Icons.flashlight_off : Icons.flashlight_on),
                onPressed: () => context.read<FlashlightCubit>().toggle(),
              ),
          };
        },
      ),
    );
  }
}
