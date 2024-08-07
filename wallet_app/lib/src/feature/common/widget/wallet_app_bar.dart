import 'package:flutter/material.dart';

import '../../../domain/model/flow_progress.dart';
import 'button/icon/back_icon_button.dart';
import 'sliver_wallet_app_bar.dart';
import 'stepper_indicator.dart';

class WalletAppBar extends StatelessWidget implements PreferredSizeWidget {
  final Widget? title;
  final FlowProgress? progress;
  final Widget? leading;
  final double? leadingWidth;
  final List<Widget>? actions;
  final PreferredSizeWidget? bottom;
  final bool automaticallyImplyLeading;

  const WalletAppBar({
    this.title,
    this.leading,
    this.progress,
    this.actions,
    this.bottom,
    this.automaticallyImplyLeading = true,
    this.leadingWidth,
    super.key,
  }) : assert(
            progress == null || bottom == null,
            "Can't provide both a bottom widget and a progress value, "
            'since the progress is rendered as a bottom widget');

  @override
  Widget build(BuildContext context) {
    /// Decide if we should show the [WalletBackButton] when no [leading] widget is provided.
    final showBackButton = Navigator.of(context).canPop() && automaticallyImplyLeading;
    return AppBar(
      title: title,
      toolbarHeight: kToolbarHeight,
      actions: actions,
      leading: leading ?? (showBackButton ? const BackIconButton() : null),
      leadingWidth: leadingWidth,
      automaticallyImplyLeading: automaticallyImplyLeading,
      titleSpacing: leading == null && !showBackButton ? null : 0.0,
      bottom: bottom ?? (progress == null ? null : _buildStepper(progress!)),
    );
  }

  PreferredSizeWidget _buildStepper(FlowProgress progress) {
    return PreferredSize(
      preferredSize: const Size.fromHeight(kStepIndicatorHeight),
      child: Container(
        height: kStepIndicatorHeight,
        alignment: Alignment.center,
        child: StepperIndicator(
          currentStep: progress.currentStep,
          totalSteps: progress.totalSteps,
        ),
      ),
    );
  }

  @override
  Size get preferredSize {
    if (bottom != null) return Size.fromHeight(kToolbarHeight + bottom!.preferredSize.height);
    if (progress != null) return const Size.fromHeight(kToolbarHeight + kStepIndicatorHeight);
    return const Size.fromHeight(kToolbarHeight);
  }
}
