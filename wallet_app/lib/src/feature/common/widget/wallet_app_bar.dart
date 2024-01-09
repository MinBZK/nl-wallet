import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'sliver_wallet_app_bar.dart';
import 'stepper_indicator.dart';

class WalletAppBar extends StatelessWidget implements PreferredSizeWidget {
  final String title;
  final double? progress;
  final Widget? leading;
  final List<Widget>? actions;

  const WalletAppBar({
    required this.title,
    this.leading,
    this.progress,
    this.actions,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return AppBar(
      shape: const LinearBorder() /* hides divider */,
      title: Text(title),
      titleTextStyle: context.textTheme.displayMedium,
      centerTitle: false,
      actions: actions,
      leading: leading,
      scrolledUnderElevation: 12,
      shadowColor: context.colorScheme.shadow,
      surfaceTintColor: context.colorScheme.background,
      titleSpacing: leading == null ? null : 0.0,
      bottom: progress == null ? null : _buildStepper(progress!),
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
  Size get preferredSize => Size.fromHeight(kToolbarHeight + (progress == null ? 0 : kStepIndicatorHeight));
}
