import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../bloc/flashlight_cubit.dart';

/// Widget that relays any torch state changes back to a
/// [FlashlightCubit] so that widgets higher up in the tree
/// e.g. [QrScreenFlashToggle] can react accordingly.
class FlashlightStateProxy extends StatelessWidget {
  final Widget child;
  final MobileScannerController controller;

  const FlashlightStateProxy({
    required this.child,
    required this.controller,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<FlashlightCubit, FlashlightState>(
      listenWhen: (prev, current) => current is FlashlightToggled,
      listener: (context, state) => controller.toggleTorch(),
      child: ValueListenableBuilder(
        valueListenable: controller.torchState,
        child: child,
        builder: (context, state, child) {
          switch (state) {
            case TorchState.off:
              context.read<FlashlightCubit>().confirmState(false);
              break;
            case TorchState.on:
              context.read<FlashlightCubit>().confirmState(true);
              break;
          }
          return child!;
        },
      ),
    );
  }
}
