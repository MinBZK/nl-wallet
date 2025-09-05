import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/button_content.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'widget/pin_field.dart';
import 'widget/pin_keyboard.dart';

class PinSetupPage extends StatelessWidget {
  /// The title displayed at the top of the PIN setup page.
  final String title;

  /// Callback invoked when a PIN digit key is pressed.
  final Function(int)? onKeyPressed;

  /// Callback invoked when the backspace key is pressed.
  final VoidCallback? onBackspacePressed;

  /// Callback invoked when the backspace key is long pressed.
  final VoidCallback? onBackspaceLongPressed;

  /// The number of digits currently entered by the user.
  final int enteredDigits;

  /// Whether the PIN field should be showing an error state.
  final bool isShowingError;

  /// Callback for when the stop button is pressed. Button is hidden if null.
  final VoidCallback? onStopPressed;

  const PinSetupPage({
    required this.title,
    required this.onKeyPressed,
    required this.onBackspacePressed,
    required this.onBackspaceLongPressed,
    required this.enteredDigits,
    this.isShowingError = false,
    this.onStopPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return OrientationBuilder(
      builder: (context, orientation) {
        switch (orientation) {
          case Orientation.portrait:
            return _buildPortrait(context);
          case Orientation.landscape:
            return _buildLandscape(context);
        }
      },
    );
  }

  Widget _buildPortrait(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Expanded(child: _buildHeader(context)),
        _buildPinField(),
        const SizedBox(height: 16),
        PinKeyboard(
          onKeyPressed: onKeyPressed,
          onBackspacePressed: onBackspacePressed,
          onBackspaceLongPressed: onBackspaceLongPressed,
        ),
        SafeArea(
          child: _buildStopButton(context),
        ),
      ],
    );
  }

  Widget _buildLandscape(BuildContext context) {
    final leftSection = onStopPressed == null
        ? Expanded(child: _buildHeader(context))
        : Expanded(
            child: Column(
              children: [
                Expanded(child: _buildHeader(context)),
                SafeArea(top: false, right: false, child: _buildStopButton(context)),
              ],
            ),
          );
    final rightSection = Expanded(
      child: Column(
        children: [
          const SizedBox(height: 16),
          _buildPinField(),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 16),
              child: PinKeyboard(
                onKeyPressed: onKeyPressed,
                onBackspacePressed: onBackspacePressed,
                onBackspaceLongPressed: onBackspaceLongPressed,
              ),
            ),
          ),
        ],
      ),
    );
    return Row(
      children: [
        leftSection,
        rightSection,
      ],
    );
  }

  Widget _buildHeader(BuildContext context) {
    final bool isLandscape = context.isLandscape;
    return WalletScrollbar(
      child: SingleChildScrollView(
        padding: isLandscape
            ? const EdgeInsets.symmetric(horizontal: 24, vertical: 38)
            : const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
        child: Align(
          alignment: isLandscape ? Alignment.centerLeft : Alignment.topLeft,
          child: TitleText(title),
        ),
      ),
    );
  }

  Widget _buildPinField() {
    return PinField(
      digits: kPinDigits,
      enteredDigits: enteredDigits,
      state: isShowingError ? PinFieldState.error : PinFieldState.idle,
    );
  }

  Widget _buildStopButton(BuildContext context) {
    if (onStopPressed == null) return const SizedBox.shrink();
    return ListButton(
      mainAxisAlignment: context.isLandscape ? MainAxisAlignment.start : MainAxisAlignment.center,
      icon: const Icon(Icons.block_flipped),
      onPressed: onStopPressed,
      iconPosition: IconPosition.start,
      text: Text.rich(context.l10n.generalStop.toTextSpan(context)),
      dividerSide: DividerSide.top,
    );
  }
}
