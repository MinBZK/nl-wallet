import 'package:flutter/material.dart';

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

  const SetupSecurityPinPage({
    required this.content,
    required this.onKeyPressed,
    required this.onBackspacePressed,
    required this.enteredDigits,
    this.showInput = true,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Expanded(
          child: LayoutBuilder(builder: (context, constraints) {
            final mq = MediaQuery.of(context);
            final fitsLogoAndText = constraints.maxHeight > (_requiredHeightToShowLogo * mq.textScaleFactor);
            return Column(
              children: [
                const SizedBox(height: 48),
                if (fitsLogoAndText) const WalletLogo(size: 80),
                const SizedBox(height: 24),
                Expanded(flex: 2, child: content),
                if (showInput) PinField(digits: kPinDigits, enteredDigits: enteredDigits),
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
}
