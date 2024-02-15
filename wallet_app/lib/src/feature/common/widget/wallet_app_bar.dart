import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'button/wallet_app_bar_back_button.dart';
import 'sliver_wallet_app_bar.dart';
import 'stepper_indicator.dart';

class WalletAppBar extends StatelessWidget implements PreferredSizeWidget {
  final Widget? title;
  final double? progress;
  final Widget? leading;
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
    super.key,
  }) : assert(
            progress == null || bottom == null,
            'Can\'t provide both a bottom widget and a progress value, '
            'since the progress is rendered as a bottom widget');

  @override
  Widget build(BuildContext context) {
    /// Decide if we should show the [WalletBackButton] when no [leading] widget is provided.
    final showBackButton = Navigator.of(context).canPop() && automaticallyImplyLeading;
    return AppBar(
      shape: const LinearBorder() /* hides divider */,
      title: title,
      scrolledUnderElevation: 12,
      shadowColor: context.colorScheme.shadow,
      surfaceTintColor: context.colorScheme.background,
      toolbarHeight: kToolbarHeight,
      titleTextStyle: context.textTheme.displayMedium,
      centerTitle: false,
      actions: actions,
      leading: leading ?? (showBackButton ? const WalletAppBarBackButton() : null),
      automaticallyImplyLeading: automaticallyImplyLeading,
      titleSpacing: leading == null && !showBackButton ? null : 0.0,
      bottom: bottom ?? (progress == null ? null : _buildStepper(progress!)),
    );
  }

  PreferredSizeWidget _buildStepper(double progress) {
    return PreferredSize(
      preferredSize: const Size.fromHeight(kStepIndicatorHeight),
      child: Container(
        height: kStepIndicatorHeight,
        alignment: Alignment.topCenter,
        child: StepperIndicator(progress: progress),
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
