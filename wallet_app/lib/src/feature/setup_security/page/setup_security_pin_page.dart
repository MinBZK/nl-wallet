import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/wallet_logo.dart';
import '../../pin/widget/pin_field.dart';
import '../../pin/widget/pin_keyboard.dart';

const _requiredHeightToShowLogo = 230.0;

class SetupSecurityPinPage extends StatelessWidget {
  final Widget content;
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;
  final int enteredDigits;
  final bool showInput;
  final bool isShowingError;

  const SetupSecurityPinPage({
    required this.content,
    required this.onKeyPressed,
    required this.onBackspacePressed,
    required this.enteredDigits,
    this.showInput = true,
    this.isShowingError = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return OrientationBuilder(builder: (context, orientation) {
      switch (orientation) {
        case Orientation.portrait:
          return _buildPortrait();
        case Orientation.landscape:
          return _buildLandscape();
      }
    });
  }

  Widget _buildPortrait() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Expanded(
          child: LayoutBuilder(builder: (context, constraints) {
            final textScaler = context.textScaler;
            //TODO: Remove (and refactor + test) deprecated_member_use once Flutter supports non-linear font scaling
            // ignore: deprecated_member_use
            final fitsLogoAndText = constraints.maxHeight > (_requiredHeightToShowLogo * textScaler.textScaleFactor);
            return Column(
              children: [
                const SizedBox(height: 48),
                if (fitsLogoAndText) const WalletLogo(size: 80),
                if (fitsLogoAndText) const SizedBox(height: 24),
                Expanded(
                  flex: 2,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: content,
                  ),
                ),
                if (showInput) _buildPinField(),
                if (showInput) const Spacer(),
              ],
            );
          }),
        ),
        Visibility(
          visible: showInput,
          maintainSize: true,
          maintainAnimation: true,
          maintainState: true,
          child: PinKeyboard(
            onKeyPressed: onKeyPressed,
            onBackspacePressed: onBackspacePressed,
          ),
        ),
      ],
    );
  }

  Widget _buildLandscape() {
    return Row(
      children: [
        Expanded(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: content,
              ),
              if (showInput) const SizedBox(height: 16),
              if (showInput) _buildPinField(),
            ],
          ),
        ),
        if (showInput)
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 16),
              child: PinKeyboard(
                onKeyPressed: onKeyPressed,
                onBackspacePressed: onBackspacePressed,
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildPinField() {
    return PinField(
      digits: kPinDigits,
      enteredDigits: enteredDigits,
      state: isShowingError ? PinFieldState.error : PinFieldState.idle,
    );
  }
}
