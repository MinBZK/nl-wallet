import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';
import '../../pin/widget/pin_field.dart';
import '../../pin/widget/pin_keyboard.dart';

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
        const SizedBox(height: 48),
        _buildImagePlaceholder(context),
        const SizedBox(height: 24),
        Expanded(flex: 3, child: content),
        if (showInput)
          PinField(
            digits: kPinDigits,
            enteredDigits: enteredDigits,
          ),
        const Spacer(),
        if (showInput)
          PinKeyboard(
            onKeyPressed: onKeyPressed,
            onBackspacePressed: onBackspacePressed,
          ),
      ],
    );
  }

  Widget _buildImagePlaceholder(BuildContext context) {
    return Container(
      width: 80,
      height: 80,
      alignment: Alignment.center,
      decoration: const BoxDecoration(
        shape: BoxShape.circle,
        color: Color(0xFFe6e6e6),
      ),
      child: Text('Image', style: Theme.of(context).textTheme.headline4),
    );
  }
}
